package dumps

import "fmt"

type BaseScores struct {
	count  int
	scores int
}

func (sco *BaseScores) process(score int) {
	sco.scores += score
	sco.count++
}

// Report prints processing statistics.
func (sco *BaseScores) Report() {
	avg := 0.0
	if sco.count > 0 {
		avg = float64(sco.scores) / float64(sco.count)
	}

	fmt.Printf("\nProcessed %d comments with total score %d (average: %.2f)\n",
		sco.count, sco.scores, avg)
}
