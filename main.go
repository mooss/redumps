package main

import (
	"bufio"
	"fmt"
	"os"
	"redumps/dumps"
	"strings"
	"time"
)

func main() {
	if len(os.Args) < 2 {
		fmt.Fprintf(os.Stderr, "Usage: %s <command> [options]\n", os.Args[0])
		fmt.Fprintf(os.Stderr, "Commands:\n")
		fmt.Fprintf(os.Stderr, "  process - Process a reddit dump file\n")
		os.Exit(1)
	}

	switch os.Args[1] {
	case "process":
		if len(os.Args) != 3 {
			fmt.Fprintf(os.Stderr, "Usage: %s process dumpfile\n", os.Args[0])
			os.Exit(1)
		}

		filename := os.Args[2]
		runProcess(filename)
	default:
		fmt.Fprintf(os.Stderr, "Unknown command: %s\n", os.Args[1])
		fmt.Fprintf(os.Stderr, "Available commands: process\n")
		os.Exit(1)
	}
}

type processor interface {
	Process(*bufio.Scanner) error
	Report()
}

func runProcess(filename string) {
	file, err := os.Open(filename)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error opening file: %v\n", err)
		os.Exit(1)
	}
	defer file.Close()

	startTime := time.Now()
	scanner := bufio.NewScanner(file)

	var proc processor

	switch {
	case strings.HasSuffix(filename, "_comments"):
		proc = &dumps.CommentProcessor{}
	case strings.HasSuffix(filename, "_submissions"):
		proc = &dumps.SubmissionProcessor{}
	default:
		fmt.Fprintf(os.Stderr, "Error: filename must end with '_comments' or '_submissions' to determine processor type\n")
		os.Exit(1)
	}

	if err := proc.Process(scanner); err != nil {
		fmt.Fprintf(os.Stderr, "Error during processing: %v\n", err)
		os.Exit(1)
	}
	proc.Report()

	elapsed := time.Since(startTime)
	fmt.Printf("Processing time: %v\n", elapsed)
}
