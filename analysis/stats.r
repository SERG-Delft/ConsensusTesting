b1n_delay_proposal <- c(10, 0)
b1n_delay_time <- c(10, 0)
b1n_delay_rand <- c(10, 0)
b1n_priority_proposal <- c(6, 4)
b1n_priority_time <- c(3, 7)
b1n_priority_rand <- c(1, 9)

b2_delay_proposal <- c(5, 5)
b2_delay_rand <- c(3, 7)
b2_delay_time <- c(8, 2)
b2_priorities <- c(0, 10)

b3n_delay_proposal <- c(5, 5)
b3n_delay_rand <- c(7, 3)
b3n_delay_time <- c(6, 4)
b3n_priority_proposal <- c(1, 9)
b3n_priority_rand <- c(0, 10)
b3n_priority_time <- c(0, 10)

delay <- b1n_delay_proposal + b1n_delay_time + b1n_delay_rand + b2_delay_proposal + b2_delay_rand + b2_delay_time + b3n_delay_proposal + b3n_delay_rand + b3n_delay_time

priority <- b1n_priority_proposal + b1n_priority_rand + b1n_priority_time + b2_priorities + b2_priorities + b2_priorities + b3n_priority_proposal + b3n_priority_rand + b3n_priority_time

m <- matrix(c(delay, priority), 2, 2)
m
fisher.test(m)

# b1_time <- c(30, 0)
# b1_prop <- c(30, 0)
# b1_rand <- c(30, 0)
# b2_time <- c(21, 9)
# b2_prop <- c(17, 13)
# b2_rand <- c(10, 20)
# b3_time <- c(23, 7)
# b3_prop <- c(16, 14)
# b3_rand <- c(20, 10)

# time <- b2_time + b3_time
# prop <- b2_prop + b3_prop
# rand <- b2_rand + b3_rand

# m <- matrix(c(time, prop), 2, 2)
# m
# fisher.test(m)

library(tidyverse)
library(ggpubr)
library(rstatix)
library(dplyr)

bug_data <- read.csv("C:/Users/Martijn.vanMeerten/workspace/studie/Thesis/ConsensusTesting/vm-setup/experiment_data.csv")
bug_data <- bug_data[grepl("delay", bug_data$Configuration, fixed = TRUE), ]
bug_data

bug_data <- bug_data %>%
  convert_as_factor(Concatenated, Configuration)
head(bug_data, 3)

counts_per_bug = bug_data %>%
  group_by(Configuration, Bug) %>%
  summarise(count = sum(Found), total = n()) %>%
  mutate(avg = count / total)

time_per_b1 <- bug_data[bug_data$Bug == "B1", ]
time_per_b1

ggboxplot(time_per_b1, x = "Configuration", y = "Time", add = "jitter")

# counts_per_bug
# xt <- xtabs(~ Configuration + count, data=counts_per_bug)
# prop.table(xt, margin = 1)

# bugs_found <- bug_data[bug_data$Found == 1 & bug_data$Bug == "B1" & grepl("delay", bug_data$Configuration, fixed = TRUE),]
# #bugs_found <- bug_data[bug_data$Found == 1 & bug_data$Bug == "B1" & (bug_data$Configuration == "delay-rand" | bug_data$Configuration == "delay-proposal"),]

# nrow(bugs_found)
# head(bugs_found, 31)

# ggboxplot(counts_per_bug, x = "Configuration", y = "avg", add = "jitter")
# wilcox.test(Found ~ Configuration | Bug, data = bug_data, exact = FALSE)

# # res.fried <- counts_per_bug %>% friedman.test(y=count ~ Configuration | Bug)
# friedman.test(y=counts_per_bug$total, groups = counts_per_bug$Configuration, block = counts_per_bug$Bug)
# friedman.test(y=bug_data$Found, groups = bug_data$Configuration, block = bug_data$Bug)
# library(PMCMRplus)
# frdAllPairsConoverTest(y = counts_per_bug$avg, groups = counts_per_bug$Configuration, blocks = counts_per_bug$Bug)
pairwise.wilcox.test(time_per_b1$Time, time_per_b1$Configuration, p.adj = "bonf", exact = FALSE)
# pairwise.wilcox.test(counts_per_bug$avg, counts_per_bug$Configuration, exact = FALSE)
# pairwise.wilcox.test(bug_data$Found, bug_data$Configuration, p.adj = "bonf", exact = FALSE)
# frdAllPairsConoverTest(y = bug_data$Found, groups = bug_data$Configuration, blocks = counts_per_bug$Bug)
