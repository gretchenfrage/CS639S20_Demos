
### Changes made

- I switched the `ConjugateGradients` function to use the 
  `ComputeLaplacian` implementation from `LaplaceSolver_1_3`.
- I switched `InnerProduct` to use `cblas_sdot`.
- I switched `Copy` to use `cblas_scopy`
- Created a bash script to automate testing.

### Behavior effects

The results are very close, but the last few float digits 
differ. This difference began upon switching `InnerProduct`
to use `cblas_dsot`. I suspect that this is because the 
original implementation summed into a `double` and casted
into a `float` at the end, whereas `cblas_dsot` may sum
into a `float`.

If this behavior doesn't occur on others' computers, I 
would conjecture that it may be because I am running an AMD
processor, whereas others usually don't.

### Performance effects

I ran these tests on my PC, which runs an AMD Threadripper 
1920x with 12 physical / 24 logical cores. This PC's
RAM access speed is 3200 MHz across 2 channels. It's worth 
noting that intel MKL does not advertise performance 
improvements for non-intel processors, and this is a 
non-intel processor.

Nevertheless, these optimizations did exhibit a slight 
overall performance increase.

```
>>>> with changes

real    0m28.735s
user    0m43.225s
sys     0m7.466s
>>>> without changes

real    0m29.435s
user    1m6.216s
sys     0m11.880s
```

What I find particularly interesting is that, although the 
real time spent is only marginally smaller (~1.3s shaved off),
the user time exhibits significant improvement (nearly cut 
in half). I suspect that intel MKL realized that parallelization
of these routines would exhibit diminishing returns at a
certain point, and decided to stop parallelizing at this
point. This would seem correct, because I cheapskated on the
RAM in my computer and so it has much more concurrency in CPU
than RAM.