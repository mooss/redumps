package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"time"
)

type RedditSubmission struct {
	ID          string `json:"id"`
	Title       string `json:"title"`
	Author      string `json:"author"`
	Score       int    `json:"score"`
	NumComments int    `json:"num_comments"`
}

type RedditComment struct {
	ID         string `json:"id"`
	Author     string `json:"author"`
	Body       string `json:"body"`
	Score      int    `json:"score"`
	ParentID   string `json:"parent_id"`
	LinkID     string `json:"link_id"`
	Subreddit  string `json:"subreddit"`
	CreatedUTC string `json:"created_utc"`
}

// UnmarshalJSON is a custom unmarshaller handling the fact that CreatedUTC is sometimes a string
// and sometimes a number.
func (c *RedditComment) UnmarshalJSON(data []byte) error {
	type Alias RedditComment
	temp := &struct {
		CreatedUTC any `json:"created_utc"`
		*Alias
	}{
		Alias: (*Alias)(c),
	}

	if err := json.Unmarshal(data, &temp); err != nil {
		return err
	}

	switch v := temp.CreatedUTC.(type) {
	case string:
		c.CreatedUTC = v
	case float64:
		c.CreatedUTC = fmt.Sprintf("%.0f", v)
	case int:
		c.CreatedUTC = fmt.Sprintf("%d", v)
	case nil:
		c.CreatedUTC = ""
	default: // At worse, try to coerce to string.
		c.CreatedUTC = fmt.Sprintf("%v", v)
	}

	return nil
}

func truncate(s string, maxLen int) string {
	if len(s) <= maxLen {
		return s
	}
	return s[:maxLen] + "..."
}

// detectType checks for the presence of specific fields to determine the type.
func detectType(line string) (string, error) {
	type temp struct {
		Title json.RawMessage `json:"title"`
		Body  json.RawMessage `json:"body"`
	}

	var t temp
	if err := json.Unmarshal([]byte(line), &t); err != nil {
		return "", err
	}

	if t.Title != nil {
		return "post", nil
	}
	if t.Body != nil {
		return "comment", nil
	}
	return "unknown", nil
}

func processPost(line string, postCount int) (int, error) {
	var post RedditSubmission
	if err := json.Unmarshal([]byte(line), &post); err != nil {
		return 0, err
	}
	fmt.Printf("Post #%d: %s (Score: %d)\n", postCount, post.Title, post.Score)
	return post.Score, nil
}

func processComment(line string, commentCount int) (int, error) {
	var comment RedditComment
	if err := json.Unmarshal([]byte(line), &comment); err != nil {
		return 0, err
	}
	bodyPreview := truncate(comment.Body, 50)
	fmt.Printf("Comment #%d by %s: %s (Score: %d)\n", commentCount, comment.Author, bodyPreview, comment.Score)
	return comment.Score, nil
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

	start := time.Now()
	scanner := bufio.NewScanner(input)
	postCount := 0
	commentCount := 0
	totalPostScore := 0
	totalCommentScore := 0

	for scanner.Scan() {
		line := scanner.Text()

		itemType, err := detectType(line)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error detecting type: %v\n", err)
			continue
		}

		switch itemType {
		case "post":
			score, err := processPost(line, postCount+1)
			if err != nil {
				fmt.Fprintf(os.Stderr, "Error processing post: %v\n", err)
			} else {
				postCount++
				totalPostScore += score
			}
		case "comment":
			score, err := processComment(line, commentCount+1)
			if err != nil {
				fmt.Fprintf(os.Stderr, "Error processing comment: %v\n", err)
			} else {
				commentCount++
				totalCommentScore += score
			}
		default:
			fmt.Fprintf(os.Stderr, "Unknown item type: %s\n", line)
		}
	}

	if err := scanner.Err(); err != nil {
		fmt.Fprintf(os.Stderr, "Error reading input: %v\n", err)
		os.Exit(1)
	}

	postAvg := 0.0
	if postCount > 0 {
		postAvg = float64(totalPostScore) / float64(postCount)
	}

	commentAvg := 0.0
	if commentCount > 0 {
		commentAvg = float64(totalCommentScore) / float64(commentCount)
	}

	fmt.Printf("\nProcessed %d posts with total score %d (average: %.2f)\n",
		postCount, totalPostScore, postAvg)
	fmt.Printf("Processed %d comments with total score %d (average: %.2f)\n",
		commentCount, totalCommentScore, commentAvg)

	elapsed := time.Since(start)
	fmt.Printf("Processing time: %v\n", elapsed)
}
