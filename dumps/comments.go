package dumps

import (
	"fmt"

	"github.com/buger/jsonparser"
)

///////////////
// Processor //

type CommentScores struct {
	BaseScores
}

func (sco *CommentScores) Process(data []byte) error {
	author, _ := jsonparser.GetString(data, "author")
	body, _ := jsonparser.GetString(data, "body")
	score, err := jsonparser.GetInt(data, "score")
	if err != nil {
		return err
	}

	sco.process(int(score))
	fmt.Printf(
		"Comment #%d by %s: %s (Score: %d)\n",
		sco.count, author, truncate(body, 50), score,
	)
	return nil
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
