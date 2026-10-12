# x = 10
# y = 20
# z = 3

# result = x + y * z
# print(result)

# result = (x + y) * z
# print(result)

# result = result - 10
# print(result)

# result = result / 2
# print(result)

# x = 10
# y = 20

# print(x < y)
# print(x > y)
# print(x == 10)

# x = 10

# if x > 5:
#     print(x)

# print(20)

# x = 3

# if x > 5:
#     print(x)

# print(20) 

# x = 0

# while x < 5:
#     print(x)
#     x = 1 + x

# def add(a, b):
#     return a + b

# x = add(10, 20)
# print(x)

# def find(x):
#     while x > 0:
#         if x == 3:
#             return x
#         x = x - 1

#     return 0

# print(find(10))

# message = "hello"
# print(message)

# numbers = [10, 20, 30]
# print(numbers)
# print(numbers[3])

# a = [1, 2]
# b = a

# b.append(3)

# print(a)
# print(b)

# message = "hello"
# print(message)

# def a():
#     return

# print(a())

# a = ["hello"]
# b = a

# a = None
# print(b)

# b = None

# [1, 2, 4]


# a = []
# a.append(a)
# a = None


# x = 10 + 20 * 3
# x + 5

# x = 10

# if x > 5:
#     x = x + 20

# x

# x = 3

# if x > 5:
#     x = x + 20

# x

# x = 5

# while x > 0:
#     x = x - 1

# x

# def add(a, b):
#     return a + b

# x = add(10, 20)

# x

# def add(a, b):
#     return a + b

# x = 10 + add(20, 30)

# x

# def sum(x):
#     if x == 0:
#         return 0
#     return x + sum(x - 1)

# sum(5)

# def add(a, b):
#     return a + b

# add(1, add(2, 3))

# x = "hello"

# x

# x = [1, 2, 3]
# x

# x = ["global"]

# x

# def foo(a):
#     b = ["local"]
#     return a

# foo(x)

# ["garbage"]

# x = ["alive"]
# x

# f = print
# f("hello")

# a = [1, 2]
# a.append(3)
# print(a)

# a = [1, 2]

# f = a.append

# f(3)
# f(4)

# print(a)

# a = [10, 20, 30]
# print(a[1])

# b = "hello"
# print(b[4])

# a = [10, 20, 30]
# a[1] = 100
# print(a)

# class Dog:
#     legs = 4

# dog = Dog()

# dog.name = "Rex"
# dog.legs = 3

# print(dog.name)
# print(dog.legs)

# class Dog:
#     def speak(self):
#         return self.name

# dog = Dog()
# dog.name = "Rex"

# f = dog.speak
# dog = None
# Dog = None

# print(f())

def outer():
    x = 42

    def inner():
        return x

    return inner

f = outer()
print(f())

