# serial-plotter

Plotter serial para dados de sensores feito com PyQt5 e PyQtGraph. Este script surgiu como parte do desenvolvimento de uma planta térmica para a disciplina de Sistemas de Controle 1.

### Instalação
Garanta que você tem Python 3.7 ou mais recente instalado:

```shell
python --version
```

> [!NOTE]
> [Opcional] Crie um ambiente virtual e o ative para instalação do script.

Para instalar o plotter:
```shell
cd serial-plotter/
pip install .
```

### Como usar
Para executar o `splot`, especifique a porta seria a ser usada, e qual baud rate de comunicação:
```shell
splot COM1 9600
```

Ou, caso deseje experimentar com dados aleatórios de exemplo:
```shell
splot sim 0
```

Caso deseje especificar outras opções, consulte `splot -h` para mais informações.
