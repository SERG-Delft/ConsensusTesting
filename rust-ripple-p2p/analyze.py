import os
import re

uncategorized_count = 0

print('          |CORR|INCO|UNKO')

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
    for run in runs:
        results = open('traces/' + config + '/' + run + '/results.txt').readlines()
        if results[-1] != 'done!\n':
            incomplete.append(run)
        elif results[-2] == 'reason: all committed\n':
            correct.append(run)
        else:
            uncategorized.append(run)
            uncategorized_count += 1
    print(f'{len(correct):4d}', end = '|')
    print(f'{len(incomplete):4d}', end = '|')
    print(f'{len(uncategorized):4d}', uncategorized)

print(uncategorized_count, 'uncategorized')
