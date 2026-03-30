# Documentação do Senamby

[![English](https://img.shields.io/badge/Language-English-2563eb?style=for-the-badge)](../en/index.md)
[![Português](https://img.shields.io/badge/Idioma-Portugu%C3%AAs-16a34a?style=for-the-badge)](index.md)

## Visão Geral

Senamby é uma aplicação desktop para operar plantas usando plugins reutilizáveis. Uma planta combina:

- sensores
- atuadores
- um plugin de driver
- zero ou mais plugins de controlador

A aplicação permite definir o modelo da planta, conectá-la a uma runtime, plotar valores em tempo real e importar/exportar artefatos em JSON.

## Para Quem É

- operadores que configuram plantas e executam testes
- integradores que criam drivers para dispositivos e protocolos
- engenheiros de controle que criam controladores e bindings

## Mapa da Documentação

- [Primeiros Passos](getting-started.md)
- [Conceitos Centrais](core-concepts.md)
- [Plantas](plants.md)
- [Drivers e Controladores](drivers-and-controllers.md)
- [Formato de Arquivo de Plugin](plugin-file-format.md)
- [Comportamento da Runtime](runtime-behavior.md)
- [Solução de Problemas](troubleshooting.md)

## Fluxos Principais

1. Carregar ou criar os plugins necessários
2. Criar uma planta ou importar um arquivo JSON de planta
3. Configurar variáveis, driver e bindings de controlador
4. Conectar a planta para iniciar a runtime
5. Ajustar setpoints e controladores enquanto acompanha os gráficos