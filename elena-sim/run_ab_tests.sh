#!/bin/bash
# A/B тесты для сети «Елена»
# Запускает все комбинации защиты и сохраняет результаты в results/ab_tests_<date>/

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

NODES="${NODES:-200}"
STEPS="${STEPS:-500}"
QUANTUM="${QUANTUM:-0.9}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUTPUT_DIR="results/ab_tests_${TIMESTAMP}"

echo "========================================="
echo "A/B тесты сети «Елена»"
echo "Узлов: $NODES, шагов: $STEPS, quantum: $QUANTUM"
echo "Результаты: $OUTPUT_DIR"
echo "========================================="

mkdir -p "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR/logs"

# Запуск батча (Python)
python3 run_batch.py --nodes "$NODES" --steps "$STEPS" --quantum "$QUANTUM" --output-dir "$OUTPUT_DIR"

CSV="$OUTPUT_DIR/results.csv"
if [ -f "$CSV" ]; then
    echo ""
    echo "Построение графиков..."
    python3 plot_results.py "$CSV"
    echo ""
    echo "✅ Готово. Результаты: $OUTPUT_DIR/results.csv"
    echo "   Графики: $OUTPUT_DIR/comparison_plots.png"
    echo "   Логи:    $OUTPUT_DIR/logs/"
else
    echo "⚠️ Файл $CSV не создан."
    exit 1
fi
