import matplotlib
import matplotlib.pyplot as plt
import numpy as np

def plot_results(label_data, plot_title, x_label, y_label):
	x = np.arange(len(label_data['labels']))  # the label locations
	width = 0.15  # the width of the bars

	fig, ax = plt.subplots()
	rects1 = ax.bar(x - width/2, label_data['left_means'], width, label=label_data['left_label'])
	rects2 = ax.bar(x + width/2, label_data['right_means'], width, label=label_data['right_label'])

	# Add some text for labels, title and custom x-axis tick labels, etc.
	ax.set_ylabel(y_label)
	ax.set_xlabel(x_label)
	ax.set_title(plot_title)
	ax.set_xticks(x)
	ax.set_xticklabels(label_data['labels'])
	ax.legend()

	def autolabel(rects):
	    """Attach a text label above each bar in *rects*, displaying its height."""
	    for rect in rects:
	        height = rect.get_height()
	        ax.annotate('{}'.format(height),
	                    xy=(rect.get_x() + rect.get_width() / 2, height),
	                    xytext=(0, 3),  # 3 points vertical offset
	                    textcoords="offset points",
	                    ha='center', va='bottom')


	autolabel(rects1)
	autolabel(rects2)

	fig.tight_layout()

	plt.show()

def plot_fps():
	label_data = dict(
		labels=['250x250', '500x500', '1300x1300'],
		left_label='GPU',
		left_means=[428.75, 480.66, 1464.25],
		right_label= 'CPU',
		right_means=[878.5, 3443.32, 21540.33],
	)

	plot_title = 'Rendering time per frame'
	x_label = 'Size of CA Grid'
	y_label = 'Time (μs)'

	plot_results(label_data, plot_title, x_label, y_label)

def plot_framerate():
	label_data = dict(
		labels=['250x250', '500x500', '1300x1300'],
		left_label='GPU',
		left_means=[1260.2, 580.0, 175.2],
		right_label= 'CPU',
		right_means=[1145.5, 258.75, 45.5],
	)

	plot_title = 'Frame Rate Per Second'
	x_label = 'Size of CA Grid'
	y_label = 'FPS'

	plot_results(label_data, plot_title, x_label, y_label)


if __name__ == '__main__':
	plot_fps()
	plot_framerate()