## v0.1.2 - December 30, 2024

- Fix UB detected by miri in the stacked borrows model (thanks @Imberflur!)

## v0.1.1 - May 15, 2022

- Fix a rare bug where two threads can reserve identical keys
- Fix a rare bug where free slots can be permanently lost when a key is reserved
  while an item is being removed from the arena

## v0.1.0 - November 28, 2021

Initial release
