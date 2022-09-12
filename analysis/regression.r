## Scriptie Tinus

### 0. Preparations
## 0.0. Load libraries
library(dplyr) #used for data transformations
library(tidyr) #used for data transformations
library(lme4) #used for mixed effects logistic regression
library(kableExtra) #used to output table to latex
library(ggplot2) #used to create plots
library(cowplot) #used to create plots

## 0.1. Read data
df_orig <- read.csv("C:/Users/Martijn.vanMeerten/workspace/studie/Thesis/ConsensusTesting/vm-setup/experiment_data.csv")

## 0.2. Transform data
# pivot to wide dataframe
df_wide <- 
  pivot_wider(df_orig, names_from = c("Bug"), values_from = c("Found", "Time"))
# remove priority group
df <- 
  subset(df_wide, Configuration == "delay-proposal" | 
           Configuration == "delay-rand" | 
           Configuration == "delay-time")
# set "rand" as reference category
df$Configuration <- 
  factor(df$Configuration, levels = c("delay-rand", "delay-time", "delay-proposal"))
# pivot longer for linear mixed model
df_lme <- 
  pivot_longer(df, names_to = "Bug", cols = c("Found_B1", "Found_B2", "Found_B3"))

##### ANALYSES #####
### 1. Testing tool accuracy 
## 1.1. Accuracy per bug
# logistic regression (3 times)
# dependent variable = was the bug detected? (for bug 1, 2 and 3 separately)
# independent variable = configuration
#WE KIEZEN LOGISTISCHE REGRESSIE, WANT:
#1. BINAIRE UITKOMST (JA/NEE BUG DETECTED), ANDERS KONDEN WE LINEAIRE REGRESSIE DOEN
#HIERMEE NEMEN WE OOK GEEN NORMAAL VERDEELDE DATA AAN
#2. #ZODAT WE NIET EEN GROTE GROEPSVERGELIJKING DOEN WAARBIJ JE ZEGT IS ER EEN VERSCHIL 
#TUSSEN DE DRIE GROEPEN, MAAR WE PROPOSAL EN TIME TEGEN RAND KUNNEN AFZETTEN (DAAROM)
#NIET ANOVA (--> AL DIT GELDT VOOR ALLE ANDERE LOGISTISCHE REGRESSIES OOK)
results_accuracybug1 <- 
  glm(Found_B1~Configuration, family = "binomial", data = df)
results_accuracybug2 <- 
  glm(Found_B2~Configuration, family = "binomial", data = df)
results_accuracybug3 <- 
  glm(Found_B3~Configuration, family = "binomial", data = df)

## 1.2. Accuracy across bug 1-3
# mixed effects logistic regression
# dependent variable = was the bug detected?
# independent variable = configuration
# cluster variable = bug
results_accuracy_overall <-
  glmer(value ~ Configuration + (1 | Bug), family = binomial, data = df_lme)

### 2. Time to bug detection
# linear regression
#NU WEL LINEAIRE REGRESSIE, WANT TIJD IS WEL CONTINU ALS VARIABELE
# dependent variable = time to bug
# independent variable = configuration
results_timetobug <- lm(Time_B1 ~ Configuration, data = df)

### 3. Combine results into tables

# calculate Odds Ratio's and 95% confidence interval for logistic regressions (more conentional then B's)
OR_accuracy_bug1 <- round(exp(c(coef(results_accuracybug1)[c(2,3)],
                                   confint.lm(results_accuracybug1)[c(2:3),])),3)
OR_accuracy_bug2 <- round(exp(c(coef(results_accuracybug2)[c(2,3)],
                                confint.lm(results_accuracybug2)[c(2:3),])),3)
OR_accuracy_bug3 <- round(exp(c(coef(results_accuracybug3)[c(2,3)],
                                confint.lm(results_accuracybug3)[c(2:3),])),3)
OR_accuracy_overall <- round(exp(unlist(c(coef(results_accuracy_overall)[[1]][1,c(2,3)],
                                confint(results_accuracy_overall)[c(3,4),]))),3)

# Put results into one matrix for accuracy
results_accuracy <- matrix(nrow = 8, ncol = 5) #Creates an empty matrix
results_accuracy[,1] <- 
  c("Bug 1", "", "Bug 2", "", "Bug 3", "", "Overall", "") #Denotes which model is presented
results_accuracy[,2] <- 
  c("Time", "Proposal", "Time", "Proposal", "Time", "Proposal", "Time", "Proposal") #Denotes which Configuration is compared to random
results_accuracy[,3] <- 
  c(OR_accuracy_bug1[1], OR_accuracy_bug1[2],
    OR_accuracy_bug2[1], OR_accuracy_bug2[2],
    OR_accuracy_bug3[1], OR_accuracy_bug3[2],
    OR_accuracy_overall[1], OR_accuracy_overall[2]) #OR's
results_accuracy[,4] <- 
  c(paste0(OR_accuracy_bug1[3],"-",OR_accuracy_bug1[5]),
    paste0(OR_accuracy_bug1[4],"-",OR_accuracy_bug1[6]),
    paste0(OR_accuracy_bug2[3],"-",OR_accuracy_bug2[5]),
    paste0(OR_accuracy_bug2[4],"-",OR_accuracy_bug2[6]),
    paste0(OR_accuracy_bug3[3],"-",OR_accuracy_bug3[5]),
    paste0(OR_accuracy_bug3[4],"-",OR_accuracy_bug3[6]),
    paste0(OR_accuracy_overall[3],"-",OR_accuracy_overall[5]),
    paste0(OR_accuracy_overall[4],"-",OR_accuracy_overall[6])) #95% CI's
results_accuracy[,5] <- 
  c(round(summary(results_accuracybug1)$coefficients[c(2,3),4],3), 
    round(summary(results_accuracybug2)$coefficients[c(2,3),4],3),
    round(summary(results_accuracybug3)$coefficients[c(2,3),4],3),
    round(summary(results_accuracy_overall)$coefficients[c(2,3),4],3)) #p-values

# Put results into one matrix for time
results_time <- matrix(nrow = 2, ncol = 4) #Creates an empty matrix
results_time[,1] <- 
  c("Time", "Proposal")
results_time[,2] <- 
  round(summary(results_timetobug)[[4]][c(2:3), 1], 3)
results_time[,3] <- 
  round(summary(results_timetobug)[[4]][c(2:3), 2],3)
results_time[,4] <- 
  round(summary(results_timetobug)[[4]][c(2:3), 4],3)


# Output tables
results_accuracy %>%
  kbl(caption="Accuracy of delay-proposal and delay-time in comparison with delay-rand",
      col.names = c("Bug", "Configuration", "Odds Ratio", "95% CI", "p-value"),
      align="r", format = "latex") %>%
  kable_classic(full_width = F) %>%
  kable_styling(latex_options = "HOLD_position")

results_time %>%
  kbl(caption="Time until bug detection of delay-proposal and delay-time in comparison with delay-rand in detection of Bug 1",
      col.names = c("Configuration", "B", "S.E.", "p-value"),
      align="r", format = "latex") %>%
  kable_classic(full_width = F) %>%
  kable_styling(latex_options = "HOLD_position")

### 4. Create plots
# adapt 0/1 to yes/no in variable defining whether the bug was found
df$Found_B1_adj <- as.factor(ifelse(df$Found_B1 == 1, "yes", "no"))
df$Found_B2_adj <- as.factor(ifelse(df$Found_B2 == 1, "yes", "no"))
df$Found_B3_adj <- as.factor(ifelse(df$Found_B3 == 1, "yes", "no"))
df_lme$Found_overall_adj <- as.factor(ifelse(df_lme$value == 1, "yes", "no"))

plot_accuracy_b1 <- ggplot(data = df) + 
  geom_bar(aes(Found_B1_adj, fill = Configuration), position = "dodge") +
  ggtitle("A. B1") +
  xlab("Was the bug found?") +
  ylab("Number of times") +
  theme_light() +
  theme(text = element_text(family = "serif"),
        legend.position = "bottom")

plot_accuracy_b1

plot_accuracy_b2 <- ggplot(data = df) + 
  geom_bar(aes(Found_B2_adj, fill = Configuration), position = "dodge") +
  ggtitle("B. B2") +
  xlab("Was the bug found?") +
  ylab("Number of times") +
  theme_light() +
  theme(text = element_text(family = "serif"),
        legend.position = "bottom")

plot_accuracy_b3 <- ggplot(data = df) + 
  geom_bar(aes(Found_B3_adj, fill = Configuration), position = "dodge") +
  ggtitle("C. B3") +
  xlab("Was the bug found?") +
  ylab("Number of times") +
  theme_light() +
  theme(text = element_text(family = "serif"),
        legend.position = "bottom")

plot_accuracy_overall <- ggplot(data = df_lme) + 
  geom_bar(aes(Found_overall_adj, fill = Configuration), position = "dodge") +
  ggtitle("D. Overall") +
  xlab("Was the bug found?") +
  ylab("Number of times") +
  theme_light() +
  theme(text = element_text(family = "serif"),
        legend.position = "bottom")

# combine all accuracy plots
accuracy_plots <- 
  plot_grid(plot_accuracy_b1, plot_accuracy_b2, 
            plot_accuracy_b3, plot_accuracy_overall, ncol = 2)

figure1 <- plot_grid(accuracy_plots, ncol = 1, rel_heights = c(0.1,1))

# create time plot
plot_time <- ggplot(data = df) + 
  geom_boxplot(aes(Configuration, Time_B1, fill = Configuration)) +
  xlab("Configuration") +
  ylab("Time to bug detection") +
  theme_light() +
  theme(text = element_text(family = "serif"),
        legend.position = "none")

figure2 <- plot_grid(plot_title2, plot_time, ncol = 1, rel_heights = c(0.1,1))

# save plots
ggsave("C:/Users/Martijn.vanMeerten/workspace/studie/Thesis/ConsensusTesting/documents/images/Accuracy.png", 
       figure1, width = 10, height = 7.5)
ggsave("C:/Users/Martijn.vanMeerten/workspace/studie/Thesis/ConsensusTesting/documents/images/Efficiency.png", 
       figure2, width = 8, height = 5)
