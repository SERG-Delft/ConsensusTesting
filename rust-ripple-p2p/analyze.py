import os
import re
import sys

if len(sys.argv) == 3:
    if sys.argv[1] == 'show':
        for config in os.listdir('traces'):
            if os.path.exists('traces/' + config + '/' + sys.argv[2]):
                print(open('traces/' + config + '/' + sys.argv[2] + '/results.txt').read(), end = '')
                exit(0)

uncategorized_count = 0

def neg(x):
    return lambda x: not x

def insufficient_support_l(line: str):
    return bool(re.search('[flag] Timeout after \d+ messages\n', line)) or \
        bool(re.search('[flag] Ledger \d+ diverged and has insufficient support\n', line))

print('          |TOTAL|CORRECT|INSUFFICIENT|INCOMPLETE|UNCATEGORIZED')

for config in os.listdir('traces'):
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
    for run in runs:
        results = open('traces/' + config + '/' + run + '/results.txt').readlines()
        if results[-1] != 'done!\n':
            incomplete.append(run)
        elif results[-2] == 'reason: all committed\n':
            correct.append(run)
        elif results[4] == 'reason: flags\n':
            flags = results[5:-1]
            if len(list(filter(neg(insufficient_support_l), flags))) == 0:
                insufficient_support.append(run)
            else:
                uncategorized.append(run)
                uncategorized_count += 1
        else:
            uncategorized.append(run)
            uncategorized_count += 1
    print(f'{len(runs):5d}', end = '|')
    print(f'{len(correct):7d}', end = '|')
    print(f'{len(insufficient_support):12d}', end = '|')
    print(f'{len(incomplete):10d}', end = '|')
    print(f'{len(uncategorized):13d}', uncategorized)

print(uncategorized_count, 'uncategorized')
