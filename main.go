package main

import (
	"bufio"
	"flag"
	"fmt"
	"os"
	"redumps/conv"
	"redumps/dumps"
	"runtime/pprof"
	"strings"
	"time"
)

func usagef(msg ...string) {
	fmt.Fprint(os.Stderr, "Usage: ", strings.Join(msg, " "))
	os.Exit(1)
}

func main() {
	binary := os.Args[0]
	if len(os.Args) < 2 {
		fmt.Fprintf(os.Stderr, "Usage: %s <command> [options]\n", os.Args[0])
		fmt.Fprintf(os.Stderr, "Commands:\n")
		fmt.Fprintf(os.Stderr, "  stats  - Compute stats\n")
		os.Exit(1)
	}

	// Setup profiling.
	cpuProfile := flag.String("cpuprofile", "", "write cpu profile to file")
	memProfile := flag.String("memprofile", "", "write memory profile to file")
	flag.Parse()

	if *cpuProfile != "" {
		f, err := os.Create(*cpuProfile)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Could not create CPU profile: %v\n", err)
			os.Exit(1)
		}
		pprof.StartCPUProfile(f)
		defer pprof.StopCPUProfile()
	}

	if *memProfile != "" {
		defer func() {
			f, err := os.Create(*memProfile)
			if err != nil {
				fmt.Fprintf(os.Stderr, "Could not create memory profile: %v\n", err)
				return
			}
			defer f.Close()
			pprof.WriteHeapProfile(f)
		}()
	}

	var cmderr error
	command := flag.Args()[0]

	args := []string{}
	if len(flag.Args()) > 1 {
		args = flag.Args()[1:]
	}

	switch command {
	case "stats":
		if len(args) < 1 {
			usagef(binary, "stats dumpfile [dumpfile...]")
		}
		cmderr = runStats(args)
	default:
		fmt.Fprintf(os.Stderr, "Unknown command: %s\n", command)
		fmt.Fprintf(os.Stderr, "Available commands: stats, fields\n")
		os.Exit(1)
	}

	if cmderr != nil {
		fmt.Fprint(os.Stderr, "Error: ", cmderr, "\n")
		os.Exit(1)
	}
}

func chrono(collector *dumps.Collector) func() {
	start := time.Now()
	return func() {
		mib := conv.ToMiB(float64(collector.BytesProcessed))
		elapsed := time.Since(start).Seconds()
		throughput := conv.ToMiB(float64(collector.BytesProcessed) / elapsed)
		fmt.Printf("%s MiB processed in %.1fs (%s MiB/s)\n", mib, elapsed, throughput)
	}
}

type processor interface {
	Process([]byte) error
	Report()
}

// process processes files using concurrent line-based batching.
// batchSize controls how many lines are processed per goroutine call.
// Recommended: 64-256 for typical workloads.
func process(filenames []string, proc processor) error {
	collector := dumps.Collector{}
	defer chrono(&collector)()
	initialBuffer := make([]byte, 1024*1024)

	for _, filename := range filenames {
		// Use a closure to make sure files are closed as soon as they are processed.
		if err := func() error {
			file, err := os.Open(filename)
			if err != nil {
				return fmt.Errorf("error opening file: %w", err)
			}
			defer file.Close()

			// Increase initial and maximum size to handle long comments.
			// Initial buffer: 1MB, max buffer: 10MB.
			scanner := bufio.NewScanner(file)
			scanner.Buffer(initialBuffer, 10*1024*1024)

			if err := collector.Collect(scanner, proc.Process); err != nil {
				// To avoid throwing away a processing session, errors are just reported and not fatal.
				collector.ReportError(fmt.Errorf("error during processing: %w", err))
			}
			return nil
		}(); err != nil {
			return err
		}
	}

	collector.PrintErrorSummary()
	proc.Report()
	return nil
}

func runStats(filenames []string) error {
	var (
		comments int
		others   int
	)

	for _, filename := range filenames {
		switch {
		case strings.HasSuffix(filename, "_comments"):
			comments++
		default:
			others++
		}
	}

	switch {
	case comments > 0 && others == 0:
		return process(filenames, &dumps.CommentScores{})
	}

	return fmt.Errorf(
		"filenames must all end with '_comments' to determine stats type (got %d comments and %d others)",
		comments, others,
	)
}
