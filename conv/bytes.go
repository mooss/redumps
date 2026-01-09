// This file contains utilities to manipulate human-readeable byte values.
package conv

import "fmt"

const MiB = 1024 * 1024

// ToMiB converts bytes to a human-readable MiB string (“NN.N”).
// Does not include the MiB suffix.
func ToMiB(b float64) string {
	return fmt.Sprintf("%.1f", b/float64(MiB))
}
