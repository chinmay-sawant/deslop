package sample

import "fmt"

// Add joins values for output.
func Add(a int, b int) int {
	return a + b
}

type Worker struct{}

// Run demonstrates a method with boilerplate error handling.
func (w Worker) Run(err error) error {
	if err != nil {
		return err
	}

	fmt.Println("working")
	return nil
}
