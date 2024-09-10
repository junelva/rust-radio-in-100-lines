### rust-radio-in-100-lines
`command-line streaming internet radio in 100 lines of Rust code.`

![image](https://github.com/user-attachments/assets/d0525ff4-59a3-4527-a658-54ca14ba5836)

`cargo run --release` and enjoy the jams. Thank you to the [radiobrowser](https://www.radio-browser.info/) project for the radio stations I used during testing. This was a learning project done in preparation for a larger project, and I learned a lot about using `tokio` and managing audio streaming, buffering, decoding in my quest for smooth playback. Stay tuned for more radio goodness. 🎸

`scc .`
```
───────────────────────────────────────────────────────────────────────────────
Language                 Files     Lines   Blanks  Comments     Code Complexity
───────────────────────────────────────────────────────────────────────────────
Rust                         1       100        7         0       93         10
TOML                         1        11        1         0       10          0
───────────────────────────────────────────────────────────────────────────────
Total                        2       111        8         0      103         10
───────────────────────────────────────────────────────────────────────────────
Estimated Cost to Develop (organic) $2,483
Estimated Schedule Effort (organic) 1.41 months
Estimated People Required (organic) 0.16
───────────────────────────────────────────────────────────────────────────────
Processed 4152 bytes, 0.004 megabytes (SI)
───────────────────────────────────────────────────────────────────────────────
```
I don't know where they got 1.41 months. It took me 3 days of poking at it, with time left over for napping with cats and taking long walks on the beach. Is it perfect? No. Reviewed my code, and have advice? Open issues are welcome!
