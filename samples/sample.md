# Kaboom

## Examples of exec code
executes from the directory of the markdown file
<!--embed exec-code id="default_loc" lang="shell" -->
<!--embed-meta hash="x7aKw382RHPpIpNnCOf0PCk90HspUXFWbAf/X+Ak+rk": last_run="1697146594846" -->
```shell
ls
```
<!-- result -->
```
another.md
sample.md
test.json
```
<!--embed exec-code id="default_loc" /-->
Executes from the directory above the markdown file
<!--embed exec-code id="rel_loc" lang="shell": r_exec_path="../": cache="hash" -->
<!--embed-meta hash="6f5+iOidUyN5lgQC24+UWBlNrncqpf2loSgGwiIV8P8": last_run="1727132655402" -->
```shell
ls
```
<!-- result -->
```
Cargo.lock
Cargo.toml
README.md
embed_md
embed_md_derive
embed_md_traits
samples
target
```
<!--embed exec-code id="rel_loc" /-->

Executes from the home directory
<!--embed exec-code id="loc_expansion" lang="shell": o_lang="none": exec_path="~/" -->
<!--embed-meta hash="x7aKw382RHPpIpNnCOf0PCk90HspUXFWbAf/X+Ak+rk": last_run="1697146594855" -->
```shell
ls
```
<!-- result -->
```
```
<!--embed exec-code id="loc_expansion" /-->
Executes from the fully qualified path
<!--embed exec-code id="loc_fq" lang="shell": exec_path="/Users/efabens/code/embed-md" -->
<!--embed-meta hash="x7aKw382RHPpIpNnCOf0PCk90HspUXFWbAf/X+Ak+rk": last_run="1697146594864" -->
```shell
ls
```
List the files in the current directory
<!-- result -->
```
```
<!--embed exec-code id="loc_fq" /-->

Run some python code
<!--embed exec-code id="py-test" lang="python": cache="never" -->
<!--embed-meta hash="WbU0seJpAmlyuIOsh+zcScb95iaPemcYO5OZfFGCiGM": last_run="1697146594873" -->
```python
print([i for i in range(10) if i % 2 != 0])
```
### Results
The below lists odd numbers between 0 and 10 inclusive
<!-- result -->
```
[1, 3, 5, 7, 9]
```
<!--embed exec-code id="py-test" /-->

![](https://img.shields.io/badge/test-test-green.svg)


What
[something](https://nutshelllabs.tech)

<!--embed identity id="ident-test" color="yellow": position="start": ticket="OAK-124": clickable="true" -->
[<img src="https://img.shields.io/badge/OAK--124-In_Progress-purple" alt="A purple badge"/>](https://nutshelllabs.atlassian.net/browse/OAK-124) Structuring permissions with groups
<!--embed identity id="ident-test" /-->

<!--embed identity id="OAK-124" position="start": ticket="OAK-124": clickable="true" -->
[<img src="https://img.shields.io/badge/OAK--124-In_Progress-purple" alt="A purple badge"/>](https://nutshelllabs.atlassian.net/browse/OAK-124)permissions with groups
<!--embed identity id="OAK-124" /-->
