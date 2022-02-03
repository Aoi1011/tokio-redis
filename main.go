package main

import (
	"fmt"
	"runtime"
	"sync"
)

func main() {
	// allocate 1 logical processor for the scheduler to use
	runtime.GOMAXPROCS(1)

	var wg sync.WaitGroup
	wg.Add(2)

	fmt.Println("Start Gorountines")

	go func() {
		// Schedule the call to Done to tell main we are done
		defer wg.Done()

		// Display the alphabet 3 times
		for count := 0; count < 3; count++ {
			for char := 'A'; char < 'A'+26; char++ {
				fmt.Printf("%c ", char)
			}
		}

	}()

	go func() {
		// Schedule the call to Done to tell main we are done
		defer wg.Done()

		// Display the alphabet 3 times
		for count := 0; count < 3; count++ {
			for char := 'a'; char < 'a'+26; char++ {
				fmt.Printf("%c ", char)
			}
		}

	}()

	// Wait for the gorountines to finish
	fmt.Println("Waiting to finish")
	wg.Wait()

	fmt.Println("\nTerminating Program")
}
