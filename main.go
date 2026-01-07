package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
)

type RedditSubmission struct {
	ID          string `json:"id"`
	Title       string `json:"title"`
	Author      string `json:"author"`
	Score       int    `json:"score"`
	NumComments int    `json:"num_comments"`
}

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

	scanner := bufio.NewScanner(input)
	postCount := 0
	totalScore := 0

	for scanner.Scan() { // Trivial proof-of-concept: count posts and sum scores.
		line := scanner.Text()
		var post RedditSubmission
		if err := json.Unmarshal([]byte(line), &post); err != nil {
			fmt.Fprintf(os.Stderr, "Error parsing JSON: %v\n", err)
			continue
		}

		postCount++
		totalScore += post.Score
		fmt.Printf("Post #%d: %s (Score: %d)\n", postCount, post.Title, post.Score)
	}

	if err := scanner.Err(); err != nil {
		fmt.Fprintf(os.Stderr, "Error reading input: %v\n", err)
		os.Exit(1)
	}

	fmt.Printf("\nProcessed %d posts with total score %d (average: %.2f)\n",
		postCount, totalScore, float64(totalScore)/float64(postCount))
}
