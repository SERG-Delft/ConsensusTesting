import nltk

schedules = []


def parse_execution_file():
    with open('executions.txt', 'r') as fp:
        # print(fp.readline().strip())
        # print(fp.readline().strip())
        # print(fp.readline().strip())
        # print(fp.readline().strip())
        # print(fp.readline().strip())
        # print(fp.readline().strip())
        fp.readline()
        for i in range(50):
            execution = []
            line = fp.readline().strip()
            while line != '':
                print(line)
                execution.append(line)
                line = fp.readline().strip()

            fp.readline()

            schedules.append(execution)
            print(i)

    print(len(schedules))


def calc_edit_distances():
    edit = [[0 for _ in range(50)] for _ in range(50)]
    for i, schedule1 in enumerate(schedules):
        for j, schedule2 in enumerate(schedules):
            print(str(i) + " " + str(j))
            edit[i][j] = nltk.edit_distance(schedule1, schedule2)

    print(edit)


if __name__ == '__main__':
    parse_execution_file()
    calc_edit_distances()
