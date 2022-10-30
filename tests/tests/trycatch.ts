
let nuff = 0

try {
    nuff = 2
    throw "bla bla"
}
catch (e) {
    nuff = 1
}