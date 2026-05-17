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
