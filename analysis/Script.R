if (!require(rstatix)) {
  install.packages("rstatix")
}
if (!require(effsize)) {
  install.packages("effsize")
}
if (!require(fmsb)) {
  install.packages("fmsb")
}
if (!require(tidyverse)) {
  install.packages("tidyverse")
}

data <- read.csv("C:/Users/Martijn.vanMeerten/workspace/studie/Thesis/ConsensusTesting/vm-setup/experiment_data.csv")
data <- 
  subset(data, Configuration == "delay-proposal" |
           Configuration == "delay-rand" |
           Configuration == "delay-time")

significance_test_time <- function(dataset, bug) {
  print(paste("### RESULTS FOR ", bug, " ###"))
  bug_data = dataset[dataset$Bug == bug,]
  approaches = c("delay-proposal", "delay-time", "delay-rand")
  
  for (index1 in 1:(length(approaches) - 1)) {
    for (index2 in (index1 + 1):length(approaches)) {
      print(paste(approaches[index1], approaches[index2], sep = " vs. "))
      
      a = bug_data[bug_data$Configuration == approaches[index1], ]$Time
      b = bug_data[bug_data$Configuration == approaches[index2], ]$Time
      test <-  wilcox.test(a,
                           b,
                           exact = FALSE,
                           paired = FALSE)
      
      print(paste("p-value = ",
                  test$p.value))
      
      effect.size <- VD.A(a,b)
      
      print(paste(
        "A12 (effect size) = ",
        effect.size$estimate,
        effect.size$magnitude
      ))
      print("")
    }
  }
}

significance_test_time_time_prop <- function(dataset, bug) {
  print(paste("### RESULTS FOR ", bug, " ###"))
  bug_data = dataset[dataset$Bug == bug,]
  print("time vs. proposal")
  a = bug_data[bug_data$Configuration == "delay-time", ]$Time
  b = bug_data[bug_data$Configuration == "delay-proposal", ]$Time
  test <-  wilcox.test(a,
                        b,
                        exact = FALSE,
                        paired = FALSE)
  
  print(paste("p-value = ",
              test$p.value))
  
  effect.size <- VD.A(a,b)
  
  print(paste(
    "A12 (effect size) = ",
    effect.size$estimate,
    effect.size$magnitude
  ))
  print("")
}

significance_test_bug_found <- function(dataset, bug) {
  print(paste("### RESULTS FOR ", bug, " ###"))
  bug_data = dataset[dataset$Bug == bug,]
  approaches = c("delay-proposal", "delay-time", "delay-rand")
  
  for (index1 in 1:(length(approaches) - 1)) {
    for (index2 in (index1 + 1):length(approaches)) {
      print(paste(approaches[index1], approaches[index2], sep = " vs. "))
      
      a = bug_data[bug_data$Configuration == approaches[index1], ]$Found
      b = bug_data[bug_data$Configuration == approaches[index2], ]$Found
      
      table2 <- matrix(c(sum(a), length(a)-sum(a), sum(b), length(b)-sum(b)), nrow = 2)

      results <- fisher.test(table2)

      print(paste("p-value = ",
                  results$p.value))
      
      print(paste(
        "Odds Ratio (effect size) = ",
        results$estimate
      ))
      print("")
    }
  }
}

significance_test_found_time_prop <- function(dataset, bug) {
  print(paste("### RESULTS FOR ", bug, " ###"))
  bug_data = dataset[dataset$Bug == bug,]
  print("time vs. proposal")
  a = bug_data[bug_data$Configuration == 'delay-time', ]$Found
  b = bug_data[bug_data$Configuration == 'delay-proposal', ]$Found
  table2 <- matrix(c(sum(a), length(a)-sum(a), sum(b), length(b)-sum(b)), nrow = 2)
  results <- fisher.test(table2)

  print(paste("p-value = ",
              results$p.value))

  print(paste(
    "Odds Ratio (effect size) = ",
    results$estimate
  ))
  print("")
}

# data["Time"][data["Found"] == 0] <- 3600
data_time <- data[data$Found == 1, ]
data_time

significance_test_time(data_time, "B1")
significance_test_time(data_time, "B2")
significance_test_time(data_time, "B3")
significance_test_time_time_prop(data_time, "B1")
significance_test_time_time_prop(data_time, "B2")
significance_test_time_time_prop(data_time, "B3")


significance_test_bug_found(data, "B1")
significance_test_bug_found(data, "B2")
significance_test_bug_found(data, "B3")

significance_test_found_time_prop(data, "B1")
significance_test_found_time_prop(data, "B2")
significance_test_found_time_prop(data, "B3")
# df_wide <- 
#   pivot_wider(data, names_from = c("Bug"), values_from = c("Found", "Time"))

# data

# remove priority group
df <- data

df["Configuration"][df["Configuration"] == "delay-proposal"] <- "Proposal"
df["Configuration"][df["Configuration"] == "delay-time"] <- "Time"
df["Configuration"][df["Configuration"] == "delay-rand"] <- "Random"
df["Time"][df["Found"] == 0] <- 3600
df_time

plot_time <- ggplot(data = df_time) + 
  geom_boxplot(aes(x=Bug, Time, fill = Configuration)) +
  xlab("Bug") +
  ylab("Time to bug detection") +
  theme_light() +
  theme(text = element_text(family = "serif"),
        legend.position = "bottom")

plot_time
ggsave("C:/Users/Martijn.vanMeerten/workspace/studie/Thesis/ConsensusTesting/documents/images/Efficiency-onlyfound.png", 
       plot_time, width = 8, height = 5)