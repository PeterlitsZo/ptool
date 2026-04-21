# Resumen de la API de Lua

`ptool` expone un amplio conjunto de utilidades a través de `ptool` y `p`.

## APIs principales

- [API principal de Lua](./core.md): Ciclo de vida del script, ejecución de
  procesos, configuración y utilidades de terminal.

## Módulos

- [API de argumentos](./args.md): Análisis de esquemas de argumentos de línea
  de comandos para scripts Lua.
- [API de SemVer](./semver.md): Analiza, compara y actualiza versiones
  semánticas.
- [API de hash](./hash.md): Calcula resúmenes SHA-256, SHA-1 y MD5.
- [API de red](./net.md): Analiza URL, direcciones IP y pares host-puerto.
- [API de plataforma](./platform.md): Detecta el SO, la arquitectura y el
  target triple actuales.
- [API ANSI](./ansi.md): Construye salida de terminal con estilo mediante
  secuencias ANSI.
- [API HTTP](./http.md): Envía solicitudes HTTP y consume cuerpos de respuesta.
- [API JSON](./json.md): Analiza texto JSON y serializa valores Lua como JSON.
- [API de base de datos](./db.md): Abre conexiones de base de datos y ejecuta
  consultas SQL.
- [API SSH](./ssh.md): Conéctate a hosts remotos, ejecuta comandos y transfiere
  archivos.
- [API de rutas](./path.md): Manipula rutas de forma léxica sin tocar el
  sistema de archivos.
- [API TOML](./toml.md): Analiza, lee, actualiza y elimina valores TOML.
- [API de regex](./re.md): Compila regex y busca, captura, reemplaza o divide
  texto.
- [API de cadenas](./str.md): Recorta, divide, une, reemplaza y formatea
  cadenas.
- [API de sistema de archivos](./fs.md): Lee, escribe, crea y hace glob sobre
  rutas del sistema de archivos.
- [API de shell](./sh.md): Divide líneas de comandos con sintaxis tipo shell en
  arreglos de argumentos.
- [API de plantillas](./template.md): Renderiza plantillas de texto a partir de
  datos Lua.

Usa esta página como punto de entrada y luego salta al módulo que necesites
para ver la referencia completa de funciones.
