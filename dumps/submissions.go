package dumps

import (
	"encoding/json"
	"fmt"
	"redumps/errs"
)

type RedditSubmission struct {
	ID          string `json:"id"`
	Title       string `json:"title"`
	Author      string `json:"author"`
	Score       int    `json:"score"`
	NumComments int    `json:"num_comments"`
}

///////////////
// Processor //

type SubmissionScores struct {
	BaseScores
}

func (sco *SubmissionScores) Process(line string) error {
	var post RedditSubmission
	if err := json.Unmarshal([]byte(line), &post); err != nil {
		return errs.Prefix(err, "submission stats")
	}

	sco.process(post.Score)
	fmt.Printf("Submission #%d: %s (Score: %d)\n", sco.count, post.Title, post.Score)
	return nil
}
