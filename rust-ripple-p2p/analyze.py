import os
import re
import sys

cap = 1000000
if len(sys.argv) == 3 and sys.argv[1] == 'cap':
    cap = int(sys.argv[2])

if len(sys.argv) == 3:
    if sys.argv[1] == 'show':
        for config in os.listdir('traces'):
            if os.path.exists('traces/' + config + '/' + sys.argv[2]):
                print(open('traces/' + config + '/' + sys.argv[2] + '/results.txt').read(), end = '')
                exit(0)

uncategorized_count = 0

def f_or(x, y):
    return lambda z: x(z) or y(z)

def f_not(x):
    return lambda y: not x(y)

def f_insufficient(line):
    return bool(re.search('\[flag\] Ledger \d+ diverged and has insufficient support\n', line))

def f_timeout(line):
    return bool(re.search('\[flag\] Timeout after \d+ messages\n', line))

def f_incompatible(line):
    return bool(re.search('\[flag\] Incompatible ledger <.+>\n', line))

def count(f, flags):
    return len(list(filter(f, flags)))

print('          |TOTAL|CORRECT|INSUFFICIENT|INCOMPATIBLE|TIMEOUT|INCOMPLETE|UNCATEGORIZED')

all_timeout = []

for config in sorted(os.listdir('traces')):
    (c, d, scope) = re.search('buggy-7-(\d)-(\d)-6-(.*)-0\.2\.4', config).groups()
    if scope == 'any-scope':
        scope = 'as'
    else:
        scope = 'ss'
    print(f'c={c} d={d} {scope}|', end = '')
    runs = os.listdir('traces/' + config)
    # print('- found', len(runs), 'runs')
    correct = []
    incomplete = []
    uncategorized = []
    insufficient_support = []
    incompatible = []
    timeout = []
    for run in runs[:cap]:
        results = open('traces/' + config + '/' + run + '/results.txt').readlines()
        if results[-1] != 'done!\n':
            incomplete.append(run)
        elif results[4] == 'reason: all committed\n':
            correct.append(run)
        elif results[4] == 'reason: flags\n':
            flags = results[5:-1]
            if count(f_not(f_timeout), flags) == 0:
                timeout.append(run)
                all_timeout.append(config + '/' + run)
            elif count(f_not(f_or(f_insufficient, f_timeout)), flags) == 0 and count(f_insufficient, flags) > 0:
                insufficient_support.append(run)
            elif count(f_not(f_or(f_incompatible, f_timeout)), flags) == 0 and count(f_incompatible, flags) > 0:
                incompatible.append(run)
            elif count(f_not(f_or(f_or(f_incompatible, f_insufficient), f_timeout)), flags) == 0 and count(f_incompatible, flags) > 0 and count(f_insufficient, flags) > 0:
                insufficient_support.append(run)
                incompatible.append(run)
            else:
                uncategorized.append(run)
                uncategorized_count += 1
        else:
            uncategorized.append(run)
            uncategorized_count += 1
    print(f'{len(runs[:cap]):5d}', end = '|')
    print(f'{len(correct):7d}', end = '|')
    print(f'{len(insufficient_support):12d}', end = '|')
    print(f'{len(incompatible):12d}', end = '|')
    print(f'{len(timeout):7d}', end = '|')
    print(f'{len(incomplete):10d}', end = '|')
    print(f'{len(uncategorized):13d}', uncategorized)

print(uncategorized_count, 'uncategorized')
if len(sys.argv) == 3 and sys.argv[1] == 'list' and sys.argv[2] == 'timeouts':
    print('timeouts:', all_timeout)
