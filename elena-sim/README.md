# Симулятор сети «Елена»

Децентрализованная платёжная сеть с пост-квантовой защитой, механизмом **«Атмосферного консенсуса»** и **«Эхо-локации»** для обнаружения двойной траты без глобального консенсуса.

## Требования

- Python 3.10+
- Зависимости: `pip install -r requirements.txt`

## Запуск

```bash
cd elena-sim
pip install -r requirements.txt
python main.py --scenario 1 --nodes 100 --steps 100
```

### Сценарии

| Сценарий | Описание |
|----------|----------|
| 1 | Честная сеть |
| 2 | Классическая двойная трата |
| 3 | Квантовая двойная трата (злой узел с квантовым преимуществом) |
| 4 | Сибил-атака с квантовым усилением |

### Пример

```bash
python main.py --scenario 3 --nodes 500 --quantum 0.7 --steps 1000
```

С ожидаемым выводом: обнаружение конфликта, распространение Alert, атака предотвращена; успешных атак < 1%.

### Визуализация

```bash
python main.py --scenario 1 --nodes 100 --steps 100 --viz
```

После симуляции запустится веб-дашборд на http://127.0.0.1:8000 (граф, метрики, WebSocket).

## Структура проекта

- `core/` — узлы, транзакции, криптография, граф сети
- `simulation/` — сценарии, метрики, runner
- `visualization/` — графики, FastAPI-дашборд
- `config/` — параметры симуляции
- `tests/` — базовые тесты

## A/B батч-тесты

Запуск всех комбинаций защиты (chaff / rewiring) и сохранение результатов в CSV и графики:

```bash
# Все 8 конфигураций (baseline, no_chaff, no_rewiring, no_protection, heavy_chaff, …)
./run_ab_tests.sh

# или через Python (можно задать директорию и урезать тесты)
python3 run_batch.py --nodes 100 --steps 150
python3 run_batch.py --nodes 50 --steps 80 --max-tests 4 --output-dir results/my_run
```

Результаты сохраняются в `results/ab_tests_YYYYMMDD_HHMMSS/`:

- `results.csv` — сводная таблица (detection_time, alert_coverage, peak_load, evil_reputation_*, successful_attack, false_positives, network_diameter, avg_path_length)
- `comparison_plots.png` — сравнительные графики
- `logs/*.log` — полный вывод каждого теста

Построить только графики по уже готовому CSV:

```bash
python3 plot_results.py results/ab_tests_20240321_153045/results.csv
```

## Тесты

```bash
python -m pytest tests/ -v
# или
python tests/test_basic.py
```

## Итоги экспериментов

Механизм **«Эхо-локации»** в симуляции показал устойчивость: во всех A/B конфигурациях двойная трата обнаруживается за ~3 шага, 100% узлов получают Alert, успешных атак 0. **Рекомендуемая базовая конфигурация — без chaff** (минимальная нагрузка при той же защите). Подробнее: [RESULTS.md](RESULTS.md).

- Масштаб по умолчанию для батча: **200 узлов, 500 шагов**; опция `--scale small` (50/80) или `--scale large` (300/800).
- Сценарий 3 с **несколькими злыми узлами в сговоре:** `--scenario 3 --evil 3`.

## Лицензия

Учебный проект.
