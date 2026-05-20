import "./css/main.css";

document.querySelector<HTMLDivElement>("#app")!.innerHTML = `
  <div class="card">
    <h1>Contador BEI</h1>
    <p class="subtitle">Teste de Integração PHP + MariaDB + Vite</p>

    <div class="counter-display" id="counter">0</div>

    <button class="btn" id="incrementBtn">Aumentar Número</button>

    <div class="error-message" id="errorMsg"></div>
  </div>
`;

const counterEl = document.getElementById("counter") as HTMLDivElement;
const incrementBtn = document.getElementById(
  "incrementBtn",
) as HTMLButtonElement;
const errorMsg = document.getElementById("errorMsg") as HTMLDivElement;

const API_URL = "http://localhost:5001/api.php";

// Função para buscar o valor inicial
async function fetchCount() {
  try {
    const response = await fetch(API_URL);
    const data = await response.json();
    if (data.success) {
      counterEl.textContent = data.count.toString();
      errorMsg.textContent = "";
    } else {
      errorMsg.textContent = "Erro ao carregar do BD: " + data.error;
    }
  } catch (error) {
    errorMsg.textContent = "Servidor PHP Offline ou Inacessível.";
    console.error("Fetch error:", error);
  }
}

// Função para incrementar
async function incrementCount() {
  try {
    incrementBtn.disabled = true;
    const response = await fetch(API_URL, {
      method: "POST",
    });
    const data = await response.json();

    if (data.success) {
      counterEl.textContent = data.count.toString();

      // Animação de Bump
      counterEl.classList.remove("bump");
      void counterEl.offsetWidth; // trigger reflow
      counterEl.classList.add("bump");

      errorMsg.textContent = "";
    } else {
      errorMsg.textContent = "Erro no DB: " + data.error;
    }
  } catch (error) {
    errorMsg.textContent = "Falha de comunicação com o backend.";
    console.error("Increment error:", error);
  } finally {
    incrementBtn.disabled = false;
  }
}

incrementBtn.addEventListener("click", incrementCount);

// Inicia buscando o contador atual
fetchCount();
