# YAPS: Yet Another Plugin System

Work in progress.

Here's a snippet of what a YAPS plugin declaration currently looks like:

```rust
#[yaps_plugin]
mod adder {
    #[derive(Default)]
    pub struct Adder;

    // You can set the namespace to auto, which will set it to the struct name (Adder in this case)
    #[yaps_export(namespace = "auto")]
    impl Adder {
        // By default an exported function inherits the namespace of the impl block
        // and its id is set to its name
        fn add(&self, a: i32, b: i32) -> i32 {
            a + b
        }

        // You can then explicitly override this namespace for a single function
        #[yaps_export(namespace = "Subber", id = "sub")]
        fn sub_test(&self, a: i32, b: i32) -> i32 {
            a - b
        }
    }
}

#[yaps_plugin]
mod multiplier {
    use yaps_core::Result;

    #[derive(Default)]
    pub struct Multiplier;

    #[yaps_extern(namespace = "Adder")]
    impl Multiplier {
        async fn add(&self, a: i32, b: i32) -> i32;

        #[yaps_extern(namespace = "Subber")]
        async fn sub(&self, a: i32, b: i32) -> i32;
    }

    impl Multiplier {
        #[yaps_export(id = "mult")]
        async fn mult(&self, a: i32, b: i32) -> Result<i32> {
            let mut sum = 0;
            for _ in 0..b {
                sum = self.add(sum, a).await?;
            }
            Ok(sum)
        }

        #[yaps_export(id = "div")]
        async fn div(&self, mut a: i32, b: i32) -> Result<i32> {
            let mut i = 0;
            while a >= b {
                i = self.add(i, 1).await?;
                a = self.sub(a, b).await?;
            }
            Ok(i)
        }
    }
}
```

See `yaps-core` tests for usage example.
