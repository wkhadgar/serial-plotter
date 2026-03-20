# Solução de Problemas

[![English](https://img.shields.io/badge/Language-English-2563eb?style=for-the-badge)](../en/troubleshooting.md)
[![Português](https://img.shields.io/badge/Idioma-Portugu%C3%AAs-16a34a?style=for-the-badge)](troubleshooting.md)

## Plugin Não Encontrado

Sintomas:

- a conexão falha
- a planta abre, mas não consegue rodar

O que verificar:

- o plugin existe no workspace
- o `id` ou nome do plugin ainda corresponde ao salvo na planta
- o arquivo-fonte e o registry do plugin não foram apagados manualmente

## Controlador `pending_restart`

Significa que:

- o controlador foi salvo
- a runtime atual não consegue aplicá-lo imediatamente

Como resolver:

- reconecte a planta para reconstruir a runtime com o conjunto atualizado de plugins

## Não Foi Possível Remover Controlador Ativo

Se um controlador estiver ativo e sincronizado em uma planta rodando, a remoção é bloqueada.

Como resolver:

1. desative o controlador
2. salve a configuração
3. remova o controlador

## Problemas com Dependências Python

Se a runtime não iniciar por causa das dependências:

- revise a lista de dependências do driver/controlador
- reconecte a planta depois de corrigir o plugin
- inspecione o ambiente gerado em `Documents/Senamby/workspace/envs/`

## Planta Fechada vs Removida

Se uma planta sumiu da sessão:

- ela pode ter sido apenas fechada, e não apagada
- plantas fechadas continuam salvas e podem ser importadas/abertas novamente
- plantas removidas apagam o registry persistido do workspace
