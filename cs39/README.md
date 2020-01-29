
## [Phoenix Kahlo's](http://phoenixkahlo.com/) CLI for [sifakis/CS639S20_Demos](https://github.com/sifakis/CS639S20_Demos)

#### Installation (Mac)

```
# install gcc
brew install gcc
# install rust https://www.rust-lang.org/tools/install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# enter directory
cd cs39
# install cli
cargo install --path . --force
```

#### Usage

```
USAGE:
    cs39 [MAJOR DEMO NUMBER] [MINOR DEMO NUMBER]
        
        compile and run a demo

    cs39 --list
    
        list available demos
        
    cs39 --help
    
        print this page
        
EXAMPLE:
    cs39 0 2
    
        run LaplacianStencil_0_1
```



