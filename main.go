package main

import (
	"bufio"
	"errors"
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
		if len(os.Args) != 3 {
			usagef(os.Args[0], "process dumpfile")
		}
		cmderr = runStats(os.Args[2])
	case "fields":
		if len(os.Args) != 3 {
			usagef(os.Args[0], "fields dumpfile")
			break
		}
		cmderr = process(os.Args[2], &dumps.FieldsProcessor{})
	default:
		fmt.Fprintf(os.Stderr, "Unknown command: %s\n", os.Args[1])
		fmt.Fprintf(os.Stderr, "Available commands: stats, fields\n")
		os.Exit(1)
	}

	if cmderr != nil {
		fmt.Fprint(os.Stderr, cmderr, "\n")
		os.Exit(1)
	}
}

type processor interface {
	Process(*bufio.Scanner) error
	Report()
}

// Helper function to factor out common processing logic.
func process(filename string, proc processor) error {
	file, err := os.Open(filename)
	if err != nil {
		return fmt.Errorf("error opening file: %w", err)
	}
	defer file.Close()

	startTime := time.Now()
	scanner := bufio.NewScanner(file)

	if err := proc.Process(scanner); err != nil {
		return fmt.Errorf("error during processing: %w", err)
	}
	proc.Report()

	elapsed := time.Since(startTime)
	fmt.Printf("Processing time: %v\n", elapsed)
	return nil
}

func runStats(filename string) error {
	if strings.HasSuffix(filename, "_comments") {
		return process(filename, &dumps.CommentStats{})
	} else if strings.HasSuffix(filename, "_submissions") {
		return process(filename, &dumps.SubmissionStats{})
	}

	return errors.New("Error: filename must end with '_comments' or '_submissions' to determine stats type")
}
