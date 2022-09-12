import scipy.stats as stats

####          config1 config2
# found
# not found

b1_delay_proposal = [10, 0] # [10, 0]
b1_delay_time = [10, 0] # [10, 0]
b1_delay_rand = [10, 0] # [10, 0]
b1_priority_proposal = [6, 4] # [9, 1]
b1_priority_time = [8, 2] # [8, 2]
b1_priority_rand = [8, 2] # [7, 3]

b1n_delay_proposal = [10, 0]
b1n_delay_time = [10, 0]
b1n_delay_rand = [10, 0]
b1n_priority_proposal = [6, 4]
b1n_priority_time = [3, 7]
b1n_priority_rand = [1, 9]

b2_delay_proposal = [5, 5]
b2_delay_rand = [3, 7]
b2_delay_time = [8, 2]
b2_priorities = [0, 10]

# run 1                     # run 2
b3_delay_proposal = [10, 0] # [10, 0]
b3_delay_time = [10, 0]   # [9, 1]
b3_delay_rand = [10, 0]   # [10, 0]
b3_priority_proposal = [10, 0] #[10, 0]
b3_priority_time = [10, 0] #[10, 0]
b3_priority_rand = [6, 4] # [6, 4]

b3n_delay_proposal = [5, 5]
b3n_delay_rand = [7, 3]
b3n_delay_time = [6, 4]
b3n_priority_proposal = [1, 9]
b3n_priority_rand = [0, 10]
b3n_priority_time = [0, 10]



oddsratio, pvalue = stats.fisher_exact([b3_priority_proposal, b3_priority_rand])

print(pvalue)
print(oddsratio)
