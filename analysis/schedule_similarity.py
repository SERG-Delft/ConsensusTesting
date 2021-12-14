import nltk
import Levenshtein

schedules = []
edit = [[0 for _ in range(50)] for _ in range(50)]


def parse_execution_file():
    with open('executions0Delay.txt', 'r') as fp:
        fp.readline()
        for i in range(50):
            execution = []
            line = fp.readline().strip()
            while line != '':
                execution.append(line)
                line = fp.readline().strip()

            fp.readline()

            schedules.append(execution)
            print(i)

    print(len(schedules))


def calc_edit_ratios():
    for i, schedule1 in enumerate(schedules):
        for j, schedule2 in enumerate(schedules):
            print(str(i) + " " + str(j))
            edit[i][j] = Levenshtein.seqratio(schedule1, schedule2)

    print(edit)


def calc_edit_distances():
    for i, schedule1 in enumerate(schedules):
        for j, schedule2 in enumerate(schedules):
            print(str(i) + " " + str(j))
            edit[i][j] = nltk.edit_distance(schedule1, schedule2)

    print(edit)


result = [0 for _ in range(50)]


def calc_edit_averages():
    for i in range(50):
        sum = 0
        for j in range(50):
            if i != j:
                sum += edit[i][j]

        sum /= 49
        result[i] = sum

    total_average = 0
    for i in range(50):
        total_average += result[i]

    total_average /= 50
    print(result)
    print(total_average)


if __name__ == '__main__':
    parse_execution_file()
    calc_edit_ratios()
    calc_edit_averages()
