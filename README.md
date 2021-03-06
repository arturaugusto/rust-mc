# rust-mc
I have already gotten my feet wet in embedded development with rust by working through [japaric](https://github.com/japaric)'s [discovery](https://rust-embedded.github.io/discovery/README.html) book. The book does a good job of getting the reader to the fun stuff by doing most of the setup work. I hope to cement the lessons learned from the book and to get a more complete picture of starting an embedded project from scratch by re-implementing and expanding an [LED roulette application](https://rust-embedded.github.io/discovery/05-led-roulette/README.html).

## Tasks
_each task is marked with a tag_
- [x] Implement LED roulette
- [x] Implement serial loopback
- [x] Run LED roulette and serial loopback concurrently using `futures` directly
- [ ] Use `await!` to run LED roulette and serial loopback concurrently
