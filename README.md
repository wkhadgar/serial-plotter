# splot - Visualizador e Analisador de Controle Térmico

O **splot** é uma ferramenta para visualizar e analisar o comportamento de um sistema de controle térmico. Ele permite **selecionar controladores ativos**, ajustar suas configurações em tempo real e visualizar a resposta do sistema. Além disso, é possível alternar para a aba **Analyzer** e **analisar logs de execuções anteriores**.

### Instalação

Garanta que você tem Python 3.7 ou mais recente instalado:

```shell
python --version
```

> [!NOTE]
> [Opcional] Crie um ambiente virtual e o ative para instalação do script.

Para instalar o plotter:

```shell
cd serial-plotter/controller_framework
pip install -e .
```

## Como usar

Para executar o `splot`, utilize o seguinte comando:

```sh
python3 examples/main.py
```

## Modos

### Plotter (Execução em tempo real)

- Exibe os dados do controlador e da planta em tempo real.
- Permite selecionar e configurar diferentes controladores.
- Mostra a resposta da planta ao longo do tempo.

### Analyzer (Análise de logs)

- Permite carregar **logs de execuções anteriores**.
- Plota **temperatura e derivada dT/dt** em gráficos interativos.
- Indica **pontos críticos** (ex.: ponto de maior derivada).
- Possibilita comparar diferentes execuções.

## Funcionalidades do Plotter

- Selecionar o controlador ativo e editá-lo em tempo real.
- Ajustar parâmetros do controlador, como **Setpoint, Kp, Ki, Kd**.
- Visualizar a resposta da planta em gráficos interativos.
- Alternar entre os modos de exibição para melhor visualização.

## Funcionalidades do Analyzer
- Carregar e analisar logs de execuções anteriores.
- Fazer analise em malha aberta para calcular parâmetros de sintonia
- Fazer analise em malha fechada par analisar a resposta do sistema

## Navegação

- <kbd>Space</kbd> → Alterna entre as visões (`Plotter` e `Analyzer`).
- <kbd>Escape</kbd> → Finaliza o programa.
- **Campo de entrada** → Define a temperatura desejada (`Setpoint`) e a envia ao controlador.

## Estrutura do Projeto

```sh
splot/
├── controller_framework/
│   ├── core/                 # Lógica principal do framework
│   ├── gui/                  # Interface gráfica (Plotter e Analyzer)
│   ├── __init__.py
│   └── ...
├── examples/
│   ├── main.py               # Arquivo principal para rodar o splot
│   ├── logs/                 # Pasta com logs de execuções anteriores
├── setup.py                  # Configuração do pacote
```

## Exemplo de Uso

1. **Executar o `splot`**  
   ```sh
   python3 examples/main.py
   ```
2. **Selecionar o controlador ativo** no menu lateral.
3. **Editar os parâmetros** (Setpoint, Kp, Ki, Kd).
4. **Visualizar a resposta da planta** no gráfico.
5. **Alternar para `Analyzer`** para carregar logs anteriores.

## Planta utilizada
![Planta](https://raw.githubusercontent.com/limahigor/serial-plotter/c5f47e3c2436e8b601071a4ce413bb77daab515d/controller_framework/examples/thermal_plant.png)


[Link para o modelo 3D](https://cad.onshape.com/documents/2719c8d20779534c7559f55d/w/e520d6a9af3b32d2f18ef8f3/e/bb6b8d18dfe883fe6632567b).
