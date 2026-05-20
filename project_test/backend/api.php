<?php
header('Access-Control-Allow-Origin: *');
header('Access-Control-Allow-Methods: GET, POST');
header('Access-Control-Allow-Headers: Content-Type');
header('Content-Type: application/json');

if ($_SERVER['REQUEST_METHOD'] === 'OPTIONS') {
    exit(0);
}

// Ler a porta do mariadb a partir do bei.cfg.json na raiz do projeto
$cfgPath = __DIR__ . '/../bei.cfg.json';
$dbPort = '5002'; // Padrão
if (file_exists($cfgPath)) {
    $cfg = json_decode(file_get_contents($cfgPath), true);
    if (isset($cfg['mariadb']['port'])) {
        $dbPort = $cfg['mariadb']['port'];
    }
}

// Configurações do banco (conforme backend/.env)
$envPath = __DIR__ . '/.env';
$env = [
    'DB_HOST' => '127.0.0.1',
    'DB_PORT' => '5002',
    'DB_USERNAME' => 'root',
    'DB_PASSWORD' => 'admin',
    'DB_DATABASE' => 'bei_db',
];
if (file_exists($envPath)) {
    foreach (file($envPath, FILE_IGNORE_NEW_LINES | FILE_SKIP_EMPTY_LINES) as $line) {
        $line = trim($line);
        if ($line === '' || str_starts_with($line, '#')) {
            continue;
        }
        if (str_contains($line, '=')) {
            [$key, $val] = array_map('trim', explode('=', $line, 2));
            $env[$key] = $val;
        }
    }
}
$host = $env['DB_HOST'];
$dbPort = $env['DB_PORT'] ?? $dbPort;
$user = $env['DB_USERNAME'];
$pass = $env['DB_PASSWORD'];
$db   = $env['DB_DATABASE'];

try {
    // Adicionamos a porta à string de conexão do PDO
    $pdo = new PDO("mysql:host=$host;port=$dbPort;dbname=$db;charset=utf8mb4", $user, $pass);
    $pdo->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);

    // Inicializar tabela se não existir
    $pdo->exec("CREATE TABLE IF NOT EXISTS counter (
        id INT PRIMARY KEY,
        value INT NOT NULL DEFAULT 0
    )");

    // Garantir que a linha com id=1 existe
    $stmt = $pdo->query("SELECT value FROM counter WHERE id = 1");
    if ($stmt->rowCount() === 0) {
        $pdo->exec("INSERT INTO counter (id, value) VALUES (1, 0)");
    }

    if ($_SERVER['REQUEST_METHOD'] === 'POST') {
        $pdo->exec("UPDATE counter SET value = value + 1 WHERE id = 1");
    }

    $stmt = $pdo->query("SELECT value FROM counter WHERE id = 1");
    $row = $stmt->fetch(PDO::FETCH_ASSOC);

    echo json_encode(['success' => true, 'count' => (int)$row['value']]);

} catch (PDOException $e) {
    echo json_encode(['success' => false, 'error' => $e->getMessage()]);
}
