package main

import (
	"bufio"
	"flag"
	"fmt"
	"os"
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
		fmt.Fprintf(os.Stderr, "  fields - Count fields\n")
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
	case "fields":
		if len(args) < 1 {
			usagef(binary, "fields dumpfile [dumpfile...]")
		}
		cmderr = process(args, &dumps.FieldsProcessor{})
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

func chrono() func() {
	start := time.Now()
	return func() {
		fmt.Printf("Processing time: %v\n", time.Since(start))
	}
}

type processor interface {
	Process(string) error
	Report()
}

// Helper function to factor out common processing logic.
func process(filenames []string, proc processor) error {
	defer chrono()()

	collector := dumps.Collector{}
	initialBuffer := make([]byte, 1024*1024)

	for _, filename := range filenames {
		// Use a closure to make sure files are closed as soon as they are processed.
		if err := func() error {
			file, err := os.Open(filename)
			if err != nil {
				return fmt.Errorf("error opening file: %w", err)
			}
			defer file.Close()

			// Increase initial and maximum size to handle long comments and submissions.
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
		comments    int
		submissions int
		others      int
	)

	for _, filename := range filenames {
		switch {
		case strings.HasSuffix(filename, "_comments"):
			comments++
		case strings.HasSuffix(filename, "_submissions"):
			submissions++
		default:
			others++
		}
	}

	switch {
	case comments > 0 && submissions == 0 && others == 0:
		return process(filenames, &dumps.CommentScores{})
	case comments == 0 && submissions > 0 && others == 0:
		return process(filenames, &dumps.SubmissionScores{})
	}

	return fmt.Errorf(
		"filenames must all end with '_comments' or '_submissions' to determine stats type (got %d comments %d submissions and %d others)",
		comments, submissions, others,
	)
}
