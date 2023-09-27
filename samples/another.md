<!--embed exec-code id="default_loc" o_lang="json" -->
<!--embed-meta hash="xHB1CtK43TKj9vwTVaLCmD7dw1HWoG7vXdTzPinS+u4": last_run="1727132565194" -->
```shell
cat test.json
```
Here is what it looks like to print json
<!-- result -->
```json
{
  "test": 123
}
```
<!--embed exec-code id="default_loc" /-->
<!--embed exec-code id="date" cache="hash" -->
<!--embed-meta hash="Qm4LhI1317PXeBh9AGBW7kBiIcpWfOKETKQD3INP+PE": last_run="1720331348350" -->
```bash
date
```
blah

blah

blah

blah

<!-- result -->
```
Sat Jul  6 22:49:08 PDT 2024
```
<!--embed exec-code id="date" /-->

## Bucket Permissions

Fetch bucket permissions for all buckets in the project
<!--embed exec-code id="fetch_permissions" o_lang="json": cache="always" -->
```bash
cat test.json | jq '.test'
```
<!--embed exec-code id="fetch_permissions" /-->
