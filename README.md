# serial-plotter [thermal-plotter]

Plotter serial para dados de sensores feito com PyQt5 e PyQtGraph. Este script surgiu como parte do desenvolvimento de
uma planta térmica para a disciplina de Sistemas de Controle 1.

### Instalação

Garanta que você tem Python 3.7 ou mais recente instalado:

```shell
python --version
```

> [!IMPORTANT]
> Crie um ambiente virtual e o ative para instalação do script.

Para instalar o plotter:

```shell
cd serial-plotter/
pip install .
```

### Como usar

Para executar o thermal plotter, especifique a porta serial a ser usada, e qual baud rate da comunicação:

```shell
tp COM1 9600
```

O plotter abrirá e será possível navegar entre as visões com <kbd>Space</kbd> ou enviar uma temperatura desejada via
serial por meio da caixa de texto da visão combinada:
![image](https://github.com/user-attachments/assets/e177f56e-ef5b-4719-9aef-7c76a029f24e)


Caso deseje finalizar o programa, basta pressionar <kbd>Escape</kbd>.

Caso deseje especificar outras opções, consulte `tp -h` para mais informações.
