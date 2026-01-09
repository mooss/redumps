package main

import (
	"bufio"
	"fmt"
	"os"
	"redumps/dumps"
	"strings"
	"time"
)

func usagef(msg ...string) {
	fmt.Fprint(os.Stderr, "Usage: ", strings.Join(msg, " "))
	os.Exit(1)
}

func main() {
	if len(os.Args) < 2 {
		fmt.Fprintf(os.Stderr, "Usage: %s <command> [options]\n", os.Args[0])
		fmt.Fprintf(os.Stderr, "Commands:\n")
		fmt.Fprintf(os.Stderr, "  stats  - Compute stats\n")
		fmt.Fprintf(os.Stderr, "  fields - Count fields\n")
		os.Exit(1)
	}

	var cmderr error

	switch os.Args[1] {
	case "stats":
		if len(os.Args) < 3 {
			usagef(os.Args[0], "stats dumpfile [dumpfile...]")
		}
		cmderr = runStats(os.Args[2:])
	case "fields":
		if len(os.Args) < 3 {
			usagef(os.Args[0], "fields dumpfile [dumpfile...]")
			break
		}
		cmderr = process(os.Args[2:], &dumps.FieldsProcessor{})
	default:
		fmt.Fprintf(os.Stderr, "Unknown command: %s\n", os.Args[1])
		fmt.Fprintf(os.Stderr, "Available commands: stats, fields\n")
		os.Exit(1)
	}

	if cmderr != nil {
		fmt.Fprint(os.Stderr, "Error: ", cmderr, "\n")
		os.Exit(1)
	}
}

type processor interface {
	Process(*bufio.Scanner) error
	Report()
}

func chrono() func() {
	start := time.Now()
	return func() {
		fmt.Printf("Processing time: %v\n", time.Since(start))
	}
}

// Helper function to factor out common processing logic.
func process(filenames []string, proc processor) error {
	defer chrono()()
	for _, filename := range filenames {
		file, err := os.Open(filename)
		if err != nil {
			return fmt.Errorf("error opening file: %w", err)
		}
		scanner := bufio.NewScanner(file)
		if err := proc.Process(scanner); err != nil {
			file.Close()
			return fmt.Errorf("error during processing: %w", err)
		}
		if err := file.Close(); err != nil {
			return fmt.Errorf("error closing file: %w", err)
		}
	}

	proc.Report()
	return nil
}

func runStats(filenames []string) error {
	var (
		proc        processor
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
		proc = &dumps.CommentStats{}
	case comments == 0 && submissions > 0 && others == 0:
		proc = &dumps.SubmissionStats{}
	default:
		return fmt.Errorf("filenames must all end with '_comments' or '_submissions' to determine stats type (got %d comments %d submissions and %d others)", comments, submissions, others)
	}

	return process(filenames, proc)
}
