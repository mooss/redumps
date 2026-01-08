// Package errs implements error handling utilities.
package errs

import "fmt"

// Prefix prefixes an error with a string followed by a colon and a space, returning nil when err is
// nil.
func Prefix(err error, prefix string) error {
	if err == nil {
		return nil
	}

	return fmt.Errorf("%s: %w", prefix, err)
}
