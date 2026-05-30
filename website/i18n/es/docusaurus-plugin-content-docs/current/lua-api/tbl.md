# API de tablas

Las utilidades de tablas están disponibles en `ptool.tbl` y `p.tbl`.

Estas APIs están diseñadas para tablas tipo lista densas, con claves enteras contiguas que comienzan en `1`.

## ptool.tbl.map

> `v0.6.0` - Introducido.

`ptool.tbl.map(list, fn)` transforma cada elemento de una tabla tipo lista y devuelve una lista nueva.

- `list` (table, obligatorio): Una tabla tipo lista densa.
- `fn` (function, obligatorio): Un callback que recibe `(value, index)` y debe devolver un valor distinto de `nil`.
- Devuelve: `table`.

Comportamiento:

- `fn` se llama una vez por cada elemento, en orden.
- Si `fn` devuelve `nil`, la llamada falla en lugar de crear huecos en el resultado.
- La tabla de entrada no se modifica.

```lua
local out = p.tbl.map({ 10, 20, 30 }, function(value, index)
  return value + index
end)

print(ptool.inspect(out)) -- { 11, 22, 33 }
```

## ptool.tbl.filter

> `v0.6.0` - Introducido.

`ptool.tbl.filter(list, fn)` conserva los elementos cuyo resultado del callback sea truthy y los devuelve en una lista densa nueva.

- `list` (table, obligatorio): Una tabla tipo lista densa.
- `fn` (function, obligatorio): Un callback que recibe `(value, index)`.
- Devuelve: `table`.

Comportamiento:

- `nil` y `false` eliminan el elemento actual.
- Cualquier otro valor de Lua conserva el elemento actual.
- La tabla devuelta se reindexa desde `1`.

```lua
local out = p.tbl.filter({ "a", "bb", "ccc" }, function(value)
  return #value >= 2
end)

print(ptool.inspect(out)) -- { "bb", "ccc" }
```

## ptool.tbl.any

> `Unreleased` - Introducido.

`ptool.tbl.any(list, fn)` devuelve `true` cuando el callback produce un resultado truthy para al menos un elemento.

- `list` (table, obligatorio): Una tabla tipo lista densa.
- `fn` (function, obligatorio): Un callback que recibe `(value, index)`.
- Devuelve: `boolean`.

Comportamiento:

- `nil` y `false` se tratan como falsy. Todos los demás valores de Lua se tratan como truthy.
- La iteración se detiene en cuanto se encuentra un resultado truthy.
- Las listas vacías devuelven `false`.

```lua
local has_even = p.tbl.any({ 1, 3, 4, 5 }, function(value)
  return value % 2 == 0
end)

print(has_even) -- true
```

## ptool.tbl.all

> `Unreleased` - Introducido.

`ptool.tbl.all(list, fn)` devuelve `true` solo cuando el callback produce un resultado truthy para cada elemento.

- `list` (table, obligatorio): Una tabla tipo lista densa.
- `fn` (function, obligatorio): Un callback que recibe `(value, index)`.
- Devuelve: `boolean`.

Comportamiento:

- `nil` y `false` se tratan como falsy. Todos los demás valores de Lua se tratan como truthy.
- La iteración se detiene en cuanto se encuentra un resultado falsy.
- Las listas vacías devuelven `true`.

```lua
local all_short = p.tbl.all({ "a", "bb", "ccc" }, function(value)
  return #value <= 3
end)

print(all_short) -- true
```

## ptool.tbl.find

> `Unreleased` - Introducido.

`ptool.tbl.find(list, fn)` devuelve el primer valor original de la lista cuyo resultado del callback es truthy.

- `list` (table, obligatorio): Una tabla tipo lista densa.
- `fn` (function, obligatorio): Un callback que recibe `(value, index)`.
- Devuelve: `any | nil`.

Comportamiento:

- `nil` y `false` se tratan como falsy. Todos los demás valores de Lua se tratan como truthy.
- La iteración se detiene en la primera coincidencia.
- Las listas vacías y los casos sin coincidencias devuelven `nil`.

```lua
local found = p.tbl.find({ 10, 15, 20 }, function(value)
  return value > 12
end)

print(found) -- 15
```

## ptool.tbl.find_index

> `Unreleased` - Introducido.

`ptool.tbl.find_index(list, fn)` devuelve el índice basado en 1 del primer elemento cuyo resultado del callback es truthy.

- `list` (table, obligatorio): Una tabla tipo lista densa.
- `fn` (function, obligatorio): Un callback que recibe `(value, index)`.
- Devuelve: `integer | nil`.

Comportamiento:

- `nil` y `false` se tratan como falsy. Todos los demás valores de Lua se tratan como truthy.
- La iteración se detiene en la primera coincidencia.
- Las listas vacías y los casos sin coincidencias devuelven `nil`.

```lua
local index = p.tbl.find_index({ "cat", "dog", "eel" }, function(value)
  return value == "dog"
end)

print(index) -- 2
```

## ptool.tbl.count

> `Unreleased` - Introducido.

`ptool.tbl.count(list, fn)` cuenta cuántos elementos producen un resultado truthy en el callback.

- `list` (table, obligatorio): Una tabla tipo lista densa.
- `fn` (function, obligatorio): Un callback que recibe `(value, index)`.
- Devuelve: `integer`.

Comportamiento:

- `nil` y `false` se tratan como falsy. Todos los demás valores de Lua se tratan como truthy.
- Las listas vacías devuelven `0`.

```lua
local total = p.tbl.count({ 1, 2, 3, 4, 5 }, function(value)
  return value % 2 == 1
end)

print(total) -- 3
```

## ptool.tbl.includes

> `Unreleased` - Introducido.

`ptool.tbl.includes(list, value)` devuelve `true` cuando la lista contiene el valor dado.

- `list` (table, obligatorio): Una tabla tipo lista densa.
- `value` (any, obligatorio): El valor que se va a buscar.
- Devuelve: `boolean`.

Comportamiento:

- La lista se recorre de izquierda a derecha.
- La igualdad usa la semántica normal de Lua para `==`.
- Las listas vacías devuelven `false`.

```lua
local has_blue = p.tbl.includes({ "red", "blue", "green" }, "blue")

print(has_blue) -- true
```

## ptool.tbl.reduce

> `Unreleased` - Introducido.

`ptool.tbl.reduce(list, initial, fn)` pliega una lista en un único valor.

- `list` (table, obligatorio): Una tabla tipo lista densa.
- `initial` (any, obligatorio): El valor acumulador inicial explícito.
- `fn` (function, obligatorio): Un callback que recibe `(acc, value, index)` y debe devolver un valor distinto de `nil`.
- Devuelve: `any`.

Comportamiento:

- `initial` siempre es obligatorio, incluso para listas no vacías.
- `fn` se llama una vez por cada elemento, en orden.
- Cada resultado del callback se convierte en el siguiente valor acumulador.
- Si `fn` devuelve `nil`, la llamada genera un error en lugar de perder el estado del acumulador.
- Las listas vacías devuelven `initial` sin cambios.

```lua
local total = p.tbl.reduce({ 5, 10, 15 }, 0, function(acc, value)
  return acc + value
end)

print(total) -- 30
```

## ptool.tbl.flat_map

> `Unreleased` - Introducido.

`ptool.tbl.flat_map(list, fn)` mapea cada elemento a una tabla tipo lista densa y luego concatena esas listas devueltas en una nueva lista densa.

- `list` (table, obligatorio): Una tabla tipo lista densa.
- `fn` (function, obligatorio): Un callback que recibe `(value, index)` y debe devolver una tabla tipo lista densa.
- Devuelve: `table`.

Comportamiento:

- `fn` se llama una vez por cada elemento, en orden.
- Cada resultado del callback debe ser una tabla tipo lista densa.
- Devolver una tabla no válida o una lista no densa genera un error.
- Las listas vacías devuelven una lista vacía.

```lua
local out = p.tbl.flat_map({ 1, 2, 3 }, function(value)
  return { value, value * 10 }
end)

print(ptool.inspect(out)) -- { 1, 10, 2, 20, 3, 30 }
```

## ptool.tbl.concat

> `v0.6.0` - Introducido.

`ptool.tbl.concat(...)` concatena una o más tablas tipo lista densas en una lista nueva.

- `...` (table, obligatorio): Una o más tablas tipo lista densas.
- Devuelve: `table`.

Comportamiento:

- Los argumentos se agregan de izquierda a derecha.
- Se permiten listas vacías.
- Las tablas de entrada no se modifican.

```lua
local out = p.tbl.concat({ 1, 2 }, { 3 }, {})

print(ptool.inspect(out)) -- { 1, 2, 3 }
```

## ptool.tbl.join

> `v0.6.0` - Introducido.

`ptool.tbl.join(...)` es un alias de `ptool.tbl.concat(...)`.

- `...` (table, obligatorio): Una o más tablas tipo lista densas.
- Devuelve: `table`.

```lua
local out = p.tbl.join({ "x" }, { "y", "z" })

print(ptool.inspect(out)) -- { "x", "y", "z" }
```

## Reglas de lista

`ptool.tbl` actualmente solo admite tablas tipo lista densas.

- Las claves deben ser enteros.
- Las claves deben comenzar en `1`.
- Las claves deben ser contiguas, sin huecos.
- Las tablas dispersas y las tablas tipo diccionario generan errores.
