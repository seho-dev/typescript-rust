import "main.js"
import { stan, stoo as superStoo } from "./stanFactory"
// import { print } from "./mymodule"

const an = 0
let bu = 10 + 8

print(an)
print(bu)

if (an < bu) {
    bu = 0
}
else if (an > bu) {
    bu = 3
}
else {
    bu = 1
}

function doStuff(one: number) {
    print("stuff")
}

doStuff(1)