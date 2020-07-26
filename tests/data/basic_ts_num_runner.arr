import global as G
import js-file("./bindings/basic_ts_num") as NUM

G.print(NUM.foo(20))            # Expect 40

G.print("\n")
G.print(NUM.foo(-10))           # Expect -20

G.print("\n")
G.print(NUM.foo(-27.5))           # Expect -55


G.print("\n")
G.print(NUM.foo(99/2))           # Expect 99




G.print("\nDone\n")
