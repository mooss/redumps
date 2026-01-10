package dumps

import (
	"fmt"

	"github.com/buger/jsonparser"
)

///////////////
// Processor //

type CommentScores struct {
	count  int
	scores int
}

func (sco *CommentScores) Process(data []byte) error {
	_, _ = jsonparser.GetString(data, "author")
	_, _ = jsonparser.GetString(data, "body")
	score, err := jsonparser.GetInt(data, "score")
	if err != nil {
		return err
	}

	sco.process(int(score))
	// fmt.Printf(
	// 	"Comment #%d by %s: %s (Score: %d)\n",
	// 	sco.count, author, truncate(body, 50), score,
	// )
	return nil
}

func (sco *CommentScores) process(score int) {
	sco.scores += score
	sco.count++
}

// Report prints processing statistics.
func (sco *CommentScores) Report() {
	avg := 0.0
	if sco.count > 0 {
		avg = float64(sco.scores) / float64(sco.count)
	}

	fmt.Printf("\nProcessed %d comments with total score %d (average: %.2f)\n",
		sco.count, sco.scores, avg)
}

///////////////////////
// Utility functions //

// truncate is a utility function to shorten strings for display.
func truncate(s string, maxLen int) string {
	if len(s) <= maxLen {
		return s
	}
	return s[:maxLen] + "..."
}
