module.exports = {
    "foo": function foo(x) {
        return x * 2;
    },

    "bar": function bar(x, y) {
        return x + y;
    },

    "qux": function zoo(x) {
        return x(-100, -100);
    },

    "zoo": function qux(x) {
        return function(a1) {
            return x(a1, 100);
        }
    },

    "my_var": 9000,
};
