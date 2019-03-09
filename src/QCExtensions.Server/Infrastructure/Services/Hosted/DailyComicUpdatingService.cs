using System;
using System.Net.Http;
using System.Threading;
using System.Threading.Tasks;
using AngleSharp.Html.Dom;
using AngleSharp.Html.Parser;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using QCExtensions.Application.Extensions.DbContext;
using QCExtensions.Application.Interfaces;
using QCExtensions.Domain.Entities;

namespace QCExtensions.Server.Infrastructure.Services.Hosted
{
	public class DailyComicUpdatingService : RepeatingBackgroundService
	{
		private static HttpClient m_httpClient = new HttpClient();

		private const string QCUrl = "https://questionablecontent.net/";
		private readonly TimeZoneInfo QCTimeZone = TimeZoneInfo.FindSystemTimeZoneById(Environment.GetEnvironmentVariable("QC_TIMEZONE"));

		private readonly IDateTime _dateTime;
		private IServiceProvider _provider;
		private ILogger<DailyComicUpdatingService> _logger;

		public DailyComicUpdatingService(
			IDateTime dateTime,
			IServiceProvider serviceProvider,
			ILogger<DailyComicUpdatingService> logger)
		{
			_dateTime = dateTime;
			_provider = serviceProvider;
			_logger = logger;
		}

		protected override async Task<bool> ExecuteRepeatedlyAsync(CancellationToken stoppingToken)
		{
			var now = _dateTime.Now;
			using (var scope = _provider.CreateScope())
			{
				var context = scope.ServiceProvider.GetRequiredService<DomainDbContext>();
				var alreadyHasToday = await context.Comics.AnyAsync(c => c.PublishDate.HasValue && c.PublishDate.Value.Date == now.Date);
				if (!alreadyHasToday)
				{
					_logger.LogInformation($"We do not yet have the data for the comic on {now.Date:dddd, dd MMMM yyyy}. Fetching QC front page.");
					try
					{
						await FetchLatestComicDataAsync(context);
					}
					catch (Exception e)
					{
						_logger.LogError(e, $"Could not fetch latest comic data, exception occurred");
					}
				}
			}

			TimeSpan delay = GetTimeUntilNextUpdate(now);
			_logger.LogInformation($"Waiting for {delay} until next update.");
			await Task.Delay(delay, stoppingToken);

			return true;
		}

		private static TimeSpan GetTimeUntilNextUpdate(DateTime now)
		{
			// Check the day of week. If Saturday or Sunday, only check once (at noon)
			// Otherwise, check at these hours: 0-3,6,12,18,21-23.
			TimeSpan delay;
			if (now.DayOfWeek == DayOfWeek.Saturday)
			{
				if (now.TimeOfDay < TimeSpan.FromHours(12))
				{
					delay = TimeSpan.FromHours(12) - now.TimeOfDay;
				}
				else
				{
					delay = TimeSpan.FromHours(24) - (now.TimeOfDay - TimeSpan.FromHours(12));
				}
			}
			else if (now.DayOfWeek == DayOfWeek.Sunday)
			{
				if (now.TimeOfDay < TimeSpan.FromHours(12))
				{
					delay = TimeSpan.FromHours(12) - now.TimeOfDay;
				}
				else
				{
					delay = TimeSpan.FromHours(24) - now.TimeOfDay;
				}
			}
			else
			{
				if (now.TimeOfDay.Hours < 3)
				{
					delay = TimeSpan.FromHours(1) - (now.TimeOfDay - TimeSpan.FromHours(now.TimeOfDay.Hours));
				}
				else if (now.TimeOfDay.Hours < 6)
				{
					delay = TimeSpan.FromHours(6) - now.TimeOfDay;
				}
				else if (now.TimeOfDay.Hours < 12)
				{
					delay = TimeSpan.FromHours(12) - now.TimeOfDay;
				}
				else if (now.TimeOfDay.Hours < 18)
				{
					delay = TimeSpan.FromHours(18) - now.TimeOfDay;
				}
				else if (now.TimeOfDay.Hours < 21)
				{
					delay = TimeSpan.FromHours(21) - now.TimeOfDay;
				}
				else
				{
					delay = TimeSpan.FromHours(1) - (now.TimeOfDay - TimeSpan.FromHours(now.TimeOfDay.Hours));
				}
			}

			return delay;
		}

		private async Task FetchLatestComicDataAsync(DomainDbContext context)
		{
			var response = await m_httpClient.GetAsync($"{QCUrl}");
			if (!response.IsSuccessStatusCode)
			{
				_logger.LogWarning($"Could not fetch latest comic data, got HTTP status {response.StatusCode}");
				return;
			}
			var qcFrontPage = await response.Content.ReadAsStringAsync();

			if (string.IsNullOrEmpty(qcFrontPage))
			{
				_logger.LogWarning($"Could not fetch latest comic data, got empty response");
				return;
			}

			var parser = new HtmlParser();
			var document = parser.ParseDocument(qcFrontPage);

			var comicImageElement = document.QuerySelector("img[src*=\"/comics/\"]") as IHtmlImageElement;
			var comicImageUrl = comicImageElement.Source;
			var comicId = int.Parse(comicImageUrl.Substring(comicImageUrl.LastIndexOf('/') + 1, comicImageUrl.LastIndexOf('.') - comicImageUrl.LastIndexOf('/') - 1));


			using (var transaction = context.Database.BeginTransaction())
			{
				var (comic, wasCreated) = await context.Comics.GetOrCreateAsync(comicId);
				if (wasCreated) await context.SaveChangesAsync();

				var newsElement = document.GetElementById("newspost");
				var dateElement = newsElement?.PreviousElementSibling?.QuerySelector("b");
				if (dateElement == null)
				{
					_logger.LogWarning($"Could not fetch date for latest comic #{comicId}, couldn't find date element on page");
				}
				else
				{
					var date = DateTime.Parse(dateElement.InnerHtml);
					var utcDate = TimeZoneInfo.ConvertTimeToUtc(date, QCTimeZone);
					comic.PublishDate = utcDate;
					comic.IsAccuratePublishDate = true;
				}

				// Grab the news
				if (newsElement == null)
				{
					_logger.LogWarning($"Could not fetch news for latest comic #{comicId}, couldn't find news element on page");
				}
				else
				{
					foreach (var unwantedElements in newsElement.QuerySelectorAll("script, hr"))
					{
						unwantedElements.Remove();
					}

					var newsText = newsElement.InnerHtml
						.Replace("\r", "")
						.Replace("\n", "")
						.Replace("<br />", "<br>")
						.Replace("<br/>", "<br>")
						.Replace("<br>", "\n")
						.Trim();

					var newsEntity = await context.News.SingleOrDefaultAsync(n => n.ComicId == comicId);
					if (newsEntity == null)
					{
						// New news
						newsEntity = new News
						{
							ComicId = comicId,
							LastUpdated = DateTime.UtcNow,
							UpdateFactor = 1,
							NewsText = newsText
						};
						context.News.Add(newsEntity);
					}
					else
					{
						// Old news. Compare news text with old.
						if (newsEntity.NewsText == newsText)
						{
							_logger.LogInformation($"News text for latest comic #{comicId} hasn't changed.");
						}
						else
						{
							_logger.LogInformation($"News text for comic #{comicId} has changed. Resetting update factor and updating text.");
							newsEntity.UpdateFactor = 1;
							newsEntity.NewsText = newsText;
							newsEntity.LastUpdated = DateTime.UtcNow;
							context.News.Update(newsEntity);
						}
					}
				}

				_logger.LogInformation($"Saving any changes to the database.");
				await context.SaveChangesAsync();

				transaction.Commit();
			}
		}
	}
}