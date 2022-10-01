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

interface TheInterface {
    needHelp(nice: boolean)
}

class TheClass implements TheInterface {
    help: boolean = false

    needHelp(nice: boolean = true) {
        // return this.help
    }
}

type MyType = {
    must: string
}

type MyTypeToo = {can: boolean} | MyType

function doStuff(one: number | string) {
    print("stuff")
}

doStuff(1)