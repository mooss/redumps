package dumps

import (
	"fmt"
	"redumps/errs"

	"github.com/buger/jsonparser"
)

///////////////
// Processor //

type SubmissionScores struct {
	BaseScores
}

func (sco *SubmissionScores) Process(line string) error {
	title, err := jsonparser.GetString([]byte(line), "title")
	if err != nil && err != jsonparser.KeyPathNotFoundError {
		return errs.Prefix(err, "submission stats")
	}

	score, err := jsonparser.GetInt([]byte(line), "score")
	if err != nil {
		return err
	}

	sco.process(int(score))
	fmt.Printf("Submission #%d: %s (Score: %d)\n", sco.count, title, score)
	return nil
}
