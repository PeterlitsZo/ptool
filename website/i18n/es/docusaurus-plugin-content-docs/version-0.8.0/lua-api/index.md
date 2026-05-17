# Resumen de la API de Lua

`ptool` expone un amplio conjunto de utilidades a través de `ptool` y `p`.

Los módulos se agrupan por dominio. Dentro de cada grupo, las entradas se enumeran en orden alfabético.

## Ejecución e interacción

- [API ANSI](./ansi.md): Construye salida de terminal con estilo mediante secuencias ANSI.
- [API de argumentos](./args.md): Análisis de esquemas de argumentos de línea de comandos para scripts Lua.
- [API principal de Lua](./core.md): Ciclo de vida del script, ejecución de procesos, configuración y utilidades de terminal.
- [API de log](./log.md): Escribe logs de terminal con marcas de tiempo y nivel.
- [API de shell](./sh.md): Divide líneas de comandos con sintaxis tipo shell en arreglos de argumentos.
- [API TUI](./tui.md): Construye interfaces de terminal simples con un árbol de vistas estructurado y un bucle de eventos.

## Datos y texto

- [DateTime API](./datetime.md): analiza, compara, formatea y convierte fechas concretas con soporte de zona horaria.
- [API de hash](./hash.md): Calcula resúmenes SHA-256, SHA-1 y MD5.
- [API JSON](./json.md): Analiza texto JSON y serializa valores Lua como JSON.
- [API de regex](./re.md): Compila regex y busca, captura, reemplaza o divide texto.
- [API de SemVer](./semver.md): Analiza, compara y actualiza versiones semánticas.
- [API de cadenas](./str.md): Recorta, divide, une, reemplaza y formatea cadenas.
- [API de tablas](./tbl.md): Mapea, filtra y concatena tablas tipo lista densas.
- [API de plantillas](./template.md): Renderiza plantillas de texto a partir de datos Lua.
- [API TOML](./toml.md): Analiza, serializa, lee, actualiza y elimina valores TOML.
- [API YAML](./yaml.md): Analiza texto YAML, lee valores anidados y serializa valores Lua como YAML.
- [API Zip](./zip.md): Comprime y descomprime cadenas Lua con formatos de compresión comunes.

## Sistema de archivos y plataforma

- [API de sistema de archivos](./fs.md): Lee, escribe, crea y hace glob sobre rutas del sistema de archivos.
- [API de sistema operativo](./os.md): Lee variables de entorno del runtime e inspecciona detalles del proceso anfitrión.
- [API de rutas](./path.md): Manipula rutas de forma léxica sin tocar el sistema de archivos.
- [API de plataforma](./platform.md): Detecta el SO, la arquitectura y el target triple actuales.
- [API de procesos](./proc.md): Inspecciona procesos locales, envía señales y espera a que los procesos desaparezcan.

## Red y remoto

- [API HTTP](./http.md): Envía solicitudes HTTP y consume cuerpos de respuesta.
- [API de red](./net.md): Analiza URL, direcciones IP y pares host-puerto.
- [SSH API](./ssh.md): Conectarse a hosts remotos, ejecutar comandos, enviar solicitudes HTTP desde el host remoto y transferir archivos.

## Desarrollo y almacenamiento

- [API de base de datos](./db.md): Abre conexiones de base de datos y ejecuta consultas SQL.
- [Git API](./git.md): abre repositorios, inspecciona el estado y clona, recupera o introduce identificadores respaldados por libgit2.

Usa esta página como punto de entrada y luego salta al módulo que necesites para ver la referencia completa de funciones.
