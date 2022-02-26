# base131072

I originally made this crate in order to pack some data into tweets. However halfway through
making the crate, I discovered [with the help of a very helpful
table](https://github.com/qntm/base2048) that Twitter weights its characters, and that
Base131072 is not actually the most efficiet way to encode information on Twitter, but rather
Base2048. [Another very good crate](https://docs.rs/base2048/2.0.2/base2048) implements
Base2048.

However, this crate should still work, should you want to encode something Base131072 for some
reason!
