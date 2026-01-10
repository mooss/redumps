package dumps

import (
	"fmt"
	"sync"

	"github.com/buger/jsonparser"
)

///////////////
// Processor //

type CommentScores struct {
	mu     sync.RWMutex
	count  int
	scores int
}

func (sco *CommentScores) Process(data []byte) error {
	author, _ := jsonparser.GetString(data, "author")
	body, _ := jsonparser.GetString(data, "body")
	score, err := jsonparser.GetInt(data, "score")
	if err != nil {
		return err
	}

	sco.process(int(score) + len(author) + len(body))
	// fmt.Printf(
	// 	"Comment #%d by %s: %s (Score: %d)\n",
	// 	sco.count, author, truncate(body, 50), score,
	// )
	return nil
}

func (sco *CommentScores) process(score int) {
	sco.mu.Lock()
	defer sco.mu.Unlock()
	sco.scores += score
	sco.count++
}

// Report prints processing statistics.
func (sco *CommentScores) Report() {
	sco.mu.RLock()
	defer sco.mu.RUnlock()
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
