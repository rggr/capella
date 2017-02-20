package main

import (
	"bytes"
	"flag"
	"fmt"
	"io/ioutil"
	"log"
	"math/rand"
	"net"
	"os"
	"sync"
	"sync/atomic"
	"time"
)

const metricFileName = "sample_metrics.txt"

var (
	// Command-line flags.
	batchSize   int
	concurrency int
	capellaAddr string
	duration    time.Duration

	// A set of metrics to chose from.
	metrics map[int]string

	metricCount uint64
	wg          sync.WaitGroup
)

func init() {
	rand.Seed(time.Now().Unix())

	flag.IntVar(&batchSize, "b", 100, "the number of metrics to batch before sending")
	flag.IntVar(&concurrency, "c", 10, "the number of concurrent connections")
	flag.StringVar(&capellaAddr, "a", "127.0.0.1:8125", "the address of the capella instance with the port")
	flag.DurationVar(&duration, "d", 30*time.Second, "how long the benchmark will last")
}

func readMetricsFile() error {
	b, err := ioutil.ReadFile(metricFileName)
	if err != nil {
		return err
	}

	metrics = make(map[int]string)
	sep := []byte("\n")
	index := 0

	// We use an integer index to pick a metric at random later.
	for _, m := range bytes.Split(b, sep) {
		metric := string(m)
		if metric == "" {
			continue
		}
		metrics[index] = metric
		index += 1
	}

	return nil
}

// Get a random metric from the map that was parsed from file earlier.
func getRandomMetric() string {
	i := rand.Intn(len(metrics))
	return metrics[i]
}

// A worker function kicked off by main.
func work() {
	defer wg.Done()

	// Setup the UDP connection before starting a timer.
	conn, err := net.Dial("udp", capellaAddr)
	if err != nil {
		fmt.Fprintf(os.Stderr, "error connecting to %s: %s\n", capellaAddr, err)
		return
	}

	buffer := bytes.NewBufferString("")
	batchCounter := 0

	timer := time.After(duration)
	for {
		select {
		case <-timer:
			conn.Close()
			return
		default:
			atomic.AddUint64(&metricCount, 1)
			batchCounter += 1
			buffer.WriteString(getRandomMetric() + "\n")
			if batchCounter == batchSize {
				_, err = fmt.Fprint(conn, buffer.String())
				if err != nil {
					fmt.Fprintf(os.Stderr, "error writing to socket: %s\n", err)
				}
				batchCounter = 0
				buffer.Reset()
			}
		}
	}
}

func printSummary() {
	mc := atomic.LoadUint64(&metricCount)
	fmt.Printf("metrics sent: %d\n", mc)
	fmt.Printf("metrics per second: %.2f\n", float64(mc)/duration.Seconds())
}

func main() {
	flag.Parse()

	// Read in the sample metrics file.
	err := readMetricsFile()
	if err != nil {
		log.Fatalf("error reading in metrics file: %s\n", err)
	}

	wg.Add(concurrency)
	for i := 0; i < concurrency; i++ {
		go work()
	}
	wg.Wait()

	printSummary()
}
