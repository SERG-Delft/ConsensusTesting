import os

uncategorized_count = 0

for config in os.listdir('traces'):
    print(config)
    runs = os.listdir('traces/' + config)
    print('- found', len(runs), 'runs')
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
    print('- correct', len(correct))
    print('- incomplete', len(incomplete), incomplete)
    print('- uncategorized', len(uncategorized), uncategorized)
    print()

print(uncategorized_count, 'uncategorized')
