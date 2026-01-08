package main

import (
	"bufio"
	"fmt"
	"os"
	"redditprocessor/dumps"
	"time"
)

func main() {
	// Default to reading from stdin if no file provided.
	input := os.Stdin
	if len(os.Args) > 1 {
		file, err := os.Open(os.Args[1])
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error opening file: %v\n", err)
			os.Exit(1)
		}
		defer file.Close()
		input = file
	}

	startTime := time.Now()
	processor := dumps.Processor{}
	scanner := bufio.NewScanner(input)
	if err := processor.Process(scanner); err != nil {
		fmt.Fprintf(os.Stderr, "Error during processing: %v\n", err)
		os.Exit(1)
	}
	processor.Report()

	elapsed := time.Since(startTime)
	fmt.Printf("Processing time: %v\n", elapsed)
}
