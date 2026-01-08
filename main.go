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
	if len(os.Args) != 2 {
		fmt.Fprintf(os.Stderr, "Usage: %s dumpfile\n", os.Args[0])
		os.Exit(1)
	}

	filename := os.Args[1]
	file, err := os.Open(filename)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error opening file: %v\n", err)
		os.Exit(1)
	}
	defer file.Close()

	startTime := time.Now()
	scanner := bufio.NewScanner(file)

	if strings.Contains(filename, "_comments") {
		processor := &dumps.CommentProcessor{}
		if err := processor.Process(scanner); err != nil {
			fmt.Fprintf(os.Stderr, "Error during processing: %v\n", err)
			os.Exit(1)
		}
		processor.Report()
	} else if strings.Contains(filename, "_submissions") {
		processor := &dumps.SubmissionProcessor{}
		if err := processor.Process(scanner); err != nil {
			fmt.Fprintf(os.Stderr, "Error during processing: %v\n", err)
			os.Exit(1)
		}
		processor.Report()
	} else {
		fmt.Fprintf(os.Stderr, "Error: filename must end with '_comments' or '_submissions' to determine processor type\n")
		os.Exit(1)
	}

	elapsed := time.Since(startTime)
	fmt.Printf("Processing time: %v\n", elapsed)
}
