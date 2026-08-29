# FAQ

## Is it production ready?

No.

## Is it native?

No. Almost all aspects of an element are controlled by the framework.

## Does it have accessibility?

Currently only windows is supported. Mac and Linux are planned.

## Does it support mobile/web?

Not officially, but it should compile and PRs are welcome to improve this.

## Why do I have to clone an Element into a separate variable before using it in a callback?

Elements are Clone, but not Copy, so it requires a clone.
Luckily, there is a Rust project goal for improving the ergonomics so that an explicit clone is not required: [rust-lang/goals#107](https://github.com/rust-lang/goals/issues/107).

## Do we use AI?

Yes, some AI is used. We try to use it sensibly. AI slop PRs will be closed.
